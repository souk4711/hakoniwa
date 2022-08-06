use nix::{
    sys::signal::Signal, sys::wait, sys::wait::WaitStatus, unistd, unistd::ForkResult, unistd::Gid,
    unistd::Pid, unistd::Uid,
};
use std::{
    collections::HashMap,
    env, fs,
    path::{Path, PathBuf},
    process,
    time::{Duration, Instant},
};

use crate::{defer, ChildProcess, FileSystem, IDMap, Limits, Mount, MountType, Namespaces};

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
    pub(crate) prog: String,                  // the path of the command to run
    pub(crate) argv: Vec<String>,             // holds command line arguments
    pub(crate) envp: HashMap<String, String>, // holds env variables
    pub(crate) dir: PathBuf,                  // specifies the working directory of the process
    pub(crate) limits: Limits,
    pub(crate) namespaces: Namespaces,
    pub(crate) uid_mappings: IDMap, // User ID mappings for user namespace
    pub(crate) gid_mappings: IDMap, // Group ID mappings for user namespace
    pub(crate) hostname: String,    // hostname for uts namespace
    pub(crate) rootfs: PathBuf,     // rootfs for mount namespace
    pub(crate) mounts: Vec<Mount>,  // bind mounts for mount namespace
}

impl Executor {
    pub(crate) const EXITCODE_FAILURE: i32 = 125;

    pub fn new<SA: AsRef<str>>(prog: &str, argv: &[SA]) -> Self {
        let uid = Uid::current().as_raw();
        let gid = Gid::current().as_raw();
        Executor {
            prog: prog.to_string(),
            argv: argv.iter().map(|arg| String::from(arg.as_ref())).collect(),
            uid_mappings: IDMap {
                container_id: uid,
                host_id: uid,
                size: 1,
            },
            gid_mappings: IDMap {
                container_id: gid,
                host_id: gid,
                size: 1,
            },
            hostname: String::from("localhost"),
            rootfs: FileSystem::temp_dir(),
            ..Default::default()
        }
    }

    pub fn current_dir<P: AsRef<Path>>(&mut self, dir: P) -> &mut Self {
        self.dir = Self::_absolute_path(dir);
        self
    }

    pub fn limits(&mut self, limits: Limits) -> &mut Self {
        self.limits = limits;
        self
    }

    pub fn limit_as(&mut self, limit: Option<u64>) -> &mut Self {
        self.limits.r#as = limit;
        self
    }

    pub fn limit_core(&mut self, limit: Option<u64>) -> &mut Self {
        self.limits.core = limit;
        self
    }

    pub fn limit_cpu(&mut self, limit: Option<u64>) -> &mut Self {
        self.limits.cpu = limit;
        self
    }

    pub fn limit_fsize(&mut self, limit: Option<u64>) -> &mut Self {
        self.limits.fsize = limit;
        self
    }

    pub fn limit_nofile(&mut self, limit: Option<u64>) -> &mut Self {
        self.limits.nofile = limit;
        self
    }

    pub fn namespaces(&mut self, namespaces: Namespaces) -> &mut Self {
        self.namespaces = namespaces;
        self
    }

    pub fn mounts(&mut self, mounts: Vec<Mount>) -> &mut Self {
        self.mounts = mounts
            .into_iter()
            .filter_map(|mut mount| {
                mount.host_path = Self::_absolute_path(mount.host_path);
                match Path::new(&mount.host_path).exists() {
                    true => Some(mount),
                    false => None,
                }
            })
            .collect();
        self
    }

    pub fn share_net_ns(&mut self, value: bool) -> &mut Self {
        self.namespaces.net = Some(!value);
        self
    }

    pub fn uid(&mut self, id: u32) -> &mut Self {
        self.uid_mappings.container_id = id;
        self
    }

    pub fn gid(&mut self, id: u32) -> &mut Self {
        self.gid_mappings.container_id = id;
        self
    }

    pub fn hostname(&mut self, hostname: &str) -> &mut Self {
        self.hostname = hostname.to_string();
        self
    }

    pub fn setenv(&mut self, name: &str, value: &str) -> &mut Self {
        self.envp.insert(name.to_string(), value.to_string());
        self
    }

    pub fn bind<P1: AsRef<Path>, P2: AsRef<Path>>(&mut self, src: P1, dest: P2) -> &mut Self {
        self._bind(src, dest, MountType::Bind);
        self
    }

    pub fn ro_bind<P1: AsRef<Path>, P2: AsRef<Path>>(&mut self, src: P1, dest: P2) -> &mut Self {
        self._bind(src, dest, MountType::RoBind);
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

    fn _absolute_path<P: AsRef<Path>>(src: P) -> PathBuf {
        match src.as_ref().is_absolute() {
            true => src.as_ref().to_path_buf(),
            false => env::current_dir().unwrap_or_default().join(src),
        }
    }

    fn _bind<P1: AsRef<Path>, P2: AsRef<Path>>(&mut self, src: P1, dest: P2, r#type: MountType) {
        let src = Self::_absolute_path(src);
        self.mounts.push(Mount::new(src, dest, r#type));
    }
}
