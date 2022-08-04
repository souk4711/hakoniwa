use nix::{
    sys::signal::Signal, sys::wait, sys::wait::WaitStatus, unistd, unistd::ForkResult, unistd::Pid,
};
use std::{
    fs,
    path::{Path, PathBuf},
    process,
    time::{Duration, Instant},
};

use crate::{defer, ChildProcess, FileSystem, IDMap, Limits, Namespaces};

#[derive(Default)]
enum Status {
    #[default]
    Unset,
    Ok,                  // ok
    SandboxFailure,      // sandbox setup failure
    TimeLimitExceeded,   // time limit execeeded
    OutputLimitExceeded, // output limit exceeded
    Violation,           // syscall violation
    Signaled,            // terminated with a signal
}

pub struct Mount {
    pub(crate) source: PathBuf,
    pub(crate) target: PathBuf,
}

impl Mount {
    pub(crate) const ROOTFS_DIRS: [(&'static str, &'static str); 8] = [
        ("/bin", "bin"),     // binaries
        ("/sbin", "sbin"),   // binaries
        ("/lib", "lib"),     // libraries
        ("/lib64", "lib64"), // libraries
        ("/etc", "etc"),     // configuration
        ("/home", "home"),   // binaries, libraries, configuration
        ("/usr", "usr"),     // binaries, libraries, configuration
        ("/nix", "nix"),     // binaries, libraries, configuration -- nixpkgs
    ];
    pub(crate) const PROC_DIR: (&'static str, &'static str) = ("/proc", "proc");
    pub(crate) const WORK_DIR: (&'static str, &'static str) = ("/hakoniwa", "hakoniwa");
    pub(crate) const PUT_OLD_DIR: (&'static str, &'static str) = ("/.put_old", ".put_old");
    pub(crate) const PUT_OLD_PROC_DIR: (&'static str, &'static str) =
        ("/.put_old_proc", ".put_old_proc");
}

#[derive(Default)]
pub struct ExecutorResult {
    status: Status,
    reason: String,               // more info about the status
    exit_code: Option<i32>,       // exit code or signal number that caused an exit
    start_time: Option<Instant>,  // when process started
    finish_time: Option<Instant>, // when process finished
    real_time: Option<Duration>,  // wall time used
}

impl ExecutorResult {
    fn new() -> ExecutorResult {
        ExecutorResult {
            ..Default::default()
        }
    }
}

#[derive(Default)]
pub struct Executor {
    pub(crate) prog: String,      // the path of the command to run
    pub(crate) argv: Vec<String>, // holds command line arguments
    pub(crate) dir: PathBuf,      // specifies the working directory of the process
    pub(crate) limits: Limits,
    pub(crate) namespaces: Namespaces,
    pub(crate) uid_mappings: IDMap, // User ID mappings for user namespaces
    pub(crate) gid_mappings: IDMap, // Group ID mappings for user namespaces
    pub(crate) rootfs: PathBuf,     // rootfs for mount namespaces
    pub(crate) mounts: Vec<Mount>,  // bind mounts for mount namespaces
}

impl Executor {
    pub(crate) const EXITCODE_FAILURE: i32 = 125;

    pub fn new<T: AsRef<str>>(prog: &str, argv: &[T]) -> Self {
        let rootfs = FileSystem::temp_dir();
        let mounts = Mount::ROOTFS_DIRS
            .iter()
            .filter_map(|(source, target)| {
                if Path::new(source).exists() {
                    Some(Mount {
                        source: PathBuf::from(source),
                        target: rootfs.join(target),
                    })
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        Executor {
            prog: prog.to_string(),
            argv: argv.iter().map(|arg| String::from(arg.as_ref())).collect(),
            uid_mappings: IDMap {
                container_id: 0,
                host_id: u32::from(unistd::Uid::current()),
                size: 1,
            },
            gid_mappings: IDMap {
                container_id: 0,
                host_id: u32::from(unistd::Gid::current()),
                size: 1,
            },
            rootfs,
            mounts,
            ..Default::default()
        }
    }

    pub fn current_dir<P: AsRef<Path>>(&mut self, dir: P) -> &mut Self {
        self.dir = dir.as_ref().to_path_buf();
        self
    }

    pub fn limits(&mut self, limits: Limits) -> &mut Self {
        self.limits = limits;
        self
    }

    pub fn namespaces(&mut self, namespaces: Namespaces) -> &mut Self {
        self.namespaces = namespaces;
        self
    }

    pub fn run(&mut self) -> ExecutorResult {
        let mut result = ExecutorResult::new();
        self.prog = match FileSystem::find_executable_in_path(&self.prog) {
            Some(path) => match path.to_str() {
                Some(path) => path.to_string(),
                None => unreachable!(),
            },
            None => {
                let err = format!("{}: command not found", self.prog);
                return Self::set_result_with_failure(result, &err);
            }
        };

        if let Err(err) = fs::create_dir(&self.rootfs) {
            let err = format!("create dir {:?} failed: {}", self.rootfs, err);
            return Self::set_result_with_failure(result, &err);
        }
        defer! { fs::remove_dir_all(&self.rootfs) }

        result.start_time = Some(Instant::now());
        match unsafe { unistd::fork() } {
            Ok(ForkResult::Parent { child, .. }) => self.run_in_parent(result, child),
            Ok(ForkResult::Child) => self.run_in_child(),
            Err(err) => {
                let err = format!("fork failed: {}", err);
                Self::set_result_with_failure(result, &err)
            }
        }
    }

    fn run_in_parent(&self, result: ExecutorResult, child: Pid) -> ExecutorResult {
        if let Err(err) = wait::waitpid(child, None) {
            let err = format!("waitpid child failed: {}", err);
            return Self::set_result_with_failure(result, &err);
        }
        Self::set_result(result, None)
    }

    fn run_in_child(&self) -> ExecutorResult {
        if let Err(err) = ChildProcess::run(self) {
            let err = format!("hakoniwa: {}\n", err);
            unistd::write(libc::STDERR_FILENO, err.as_bytes()).ok();
            process::exit(Self::EXITCODE_FAILURE)
        }
        process::exit(0) // unreachable!
    }

    fn set_result(mut result: ExecutorResult, ws: Option<WaitStatus>) -> ExecutorResult {
        if let Some(ws) = ws {
            match ws {
                WaitStatus::Exited(_, exit_status) => {
                    result.status = Status::Ok;
                    result.reason = String::new();
                    result.exit_code = Some(exit_status);
                }
                WaitStatus::Signaled(_, signal, _) => {
                    result.status = match signal {
                        Signal::SIGKILL => Status::TimeLimitExceeded,
                        Signal::SIGXCPU => Status::TimeLimitExceeded,
                        Signal::SIGXFSZ => Status::OutputLimitExceeded,
                        Signal::SIGSYS => Status::Violation,
                        _ => Status::Signaled,
                    };
                    result.reason = format!("signaled: {}", signal);
                    result.exit_code = Some(128 + (signal as i32));
                }
                _ => {}
            }
        }

        if let Some(start_time) = result.start_time {
            let finish_time = Instant::now();
            result.finish_time = Some(finish_time);
            result.real_time = Some(finish_time.duration_since(start_time));
        }

        result
    }

    fn set_result_with_failure(mut result: ExecutorResult, reason: &str) -> ExecutorResult {
        result.status = Status::SandboxFailure;
        result.reason = reason.to_string();
        result.exit_code = Some(Self::EXITCODE_FAILURE);
        Self::set_result(result, None)
    }
}
