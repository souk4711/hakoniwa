use chrono::prelude::*;
use libseccomp::ScmpSyscall;
use nix::{
    mount::MsFlags,
    sys::signal::{self, Signal},
    sys::wait,
    unistd::{self, ForkResult, Gid, Pid, Uid},
};
use path_abs::{self, PathAbs};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs,
    os::unix::io::AsRawFd,
    path::{Path, PathBuf},
    process,
    time::Duration,
};

use crate::{
    child_process::{self as ChildProcess, result::ChildProcessResult},
    contrib, Error, IDMap, Limits, Mount, MountType, Namespaces, Result, Seccomp,
};

#[derive(Serialize, Deserialize, PartialEq, Default, Debug)]
pub enum ExecutorResultStatus {
    #[default]
    #[serde(rename = "UK")]
    Unknown,
    #[serde(rename = "OK")]
    Ok,
    #[serde(rename = "SE")]
    SandboxSetupError,
    #[serde(rename = "SIG")]
    Signaled,
    #[serde(rename = "RFE")]
    RestrictedFunction,
    #[serde(rename = "TLE")]
    TimeLimitExceeded,
    #[serde(rename = "OLE")]
    OutputLimitExceeded,
}

#[derive(Serialize, Default, Debug)]
pub struct ExecutorResult {
    pub status: ExecutorResultStatus,
    pub reason: String,                    // more info about the status
    pub exit_code: Option<i32>,            // exit code or signal number that caused an exit
    pub start_time: Option<DateTime<Utc>>, // when process started
    pub real_time: Option<Duration>,       // wall time used
    pub system_time: Option<Duration>,     // system CPU time used
    pub user_time: Option<Duration>,       // user CPU time used
    pub max_rss: Option<i64>,              // maximum resident set size (in kilobytes)
}

impl ExecutorResult {
    pub(crate) fn failure(reason: &str) -> Self {
        Self {
            status: ExecutorResultStatus::SandboxSetupError,
            reason: reason.to_string(),
            ..Default::default()
        }
    }
}

impl From<ChildProcessResult> for ExecutorResult {
    fn from(cpr: ChildProcessResult) -> Self {
        Self {
            status: cpr.status,
            reason: cpr.reason,
            exit_code: cpr.exit_code,
            start_time: cpr.start_time,
            real_time: cpr.real_time,
            system_time: cpr.system_time,
            user_time: cpr.user_time,
            max_rss: cpr.max_rss,
        }
    }
}

#[derive(Default)]
pub struct Executor {
    pub(crate) prog: String,                  // the path of the command to run
    pub(crate) argv: Vec<String>,             // holds command line arguments
    pub(crate) envp: HashMap<String, String>, // holds env variables
    pub(crate) dir: PathBuf,                  // mount 'dir' under '/hako'
    pub(crate) rootfs: PathBuf,               // .
    pub(crate) namespaces: Namespaces,        // linux namespaces
    pub(crate) limits: Limits,                // process resource limits
    pub(crate) seccomp: Option<Seccomp>,      // secure computing
    pub(crate) uid_mappings: IDMap,           // user ID mappings for user namespace
    pub(crate) gid_mappings: IDMap,           // group ID mappings for user namespace
    pub(crate) hostname: String,              // hostname for uts namespace
    pub(crate) mount_new_tmpfs: bool,         // mount a new tmpfs under '/tmp'
    pub(crate) mount_new_devfs: bool,         // mount a new devfs under '/dev'
    pub(crate) mounts: Vec<Mount>,            // bind mounts for mount namespace
    capture_stdout: bool,                     // capture stdout
    capture_stderr: bool,                     // capture stderr
}

impl Executor {
    pub const EXITCODE_FAILURE: i32 = 125;

    pub fn new<SA: AsRef<str>>(prog: &str, argv: &[SA]) -> Self {
        let uid = Uid::current().as_raw();
        let gid = Gid::current().as_raw();
        Self {
            prog: prog.to_string(),
            argv: argv.iter().map(|arg| String::from(arg.as_ref())).collect(),
            rootfs: contrib::tmpdir::random_name("hakoniwa"),
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
            hostname: String::from("hakoniwa"),
            ..Default::default()
        }
    }

    pub fn current_dir<P: AsRef<Path>>(&mut self, dir: P) -> Result<&mut Self> {
        fs::canonicalize(&dir)
            .map_err(|err| Error::PathError(dir.as_ref().to_path_buf(), err.to_string()))
            .map(|val| {
                self.dir = val;
                self
            })
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

    pub fn mount_new_tmpfs(&mut self, mount_new_tmpfs: bool) -> &mut Self {
        self.mount_new_tmpfs = mount_new_tmpfs;
        self
    }

    pub fn mount_new_devfs(&mut self, mount_new_devfs: bool) -> &mut Self {
        self.mount_new_devfs = mount_new_devfs;
        self
    }

    pub(crate) fn mounts(&mut self, mounts: &[Mount]) -> &mut Self {
        self.mounts = vec![]; // reinitialize
        for mount in mounts {
            _ = self._bind(
                mount.host_path.clone(),
                mount.container_path.clone(),
                mount.r#type.clone(),
            );
        }
        self
    }

    pub fn bind<P1: AsRef<Path>, P2: AsRef<Path>>(
        &mut self,
        src: P1,
        dest: P2,
    ) -> Result<&mut Self> {
        self._bind(src, dest, MountType::Bind)
    }

    pub fn ro_bind<P1: AsRef<Path>, P2: AsRef<Path>>(
        &mut self,
        src: P1,
        dest: P2,
    ) -> Result<&mut Self> {
        self._bind(src, dest, MountType::RoBind)
    }

    fn _bind<P1: AsRef<Path>, P2: AsRef<Path>>(
        &mut self,
        src: P1,
        dest: P2,
        r#type: MountType,
    ) -> Result<&mut Self> {
        let src = fs::canonicalize(&src)
            .map_err(|err| Error::PathError(src.as_ref().to_path_buf(), err.to_string()))?;
        let dest = PathAbs::new(&dest)
            .map_err(|err| Error::PathError(dest.as_ref().to_path_buf(), err.to_string()))?;
        self.mounts.push(Mount::new(&src, &dest, r#type));
        Ok(self)
    }

    pub fn setenv(&mut self, name: &str, value: &str) -> &mut Self {
        self.envp.insert(name.to_string(), value.to_string());
        self
    }

    pub(crate) fn limits(&mut self, limits: &Limits) -> &mut Self {
        self.limits = limits.clone();
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

    pub fn limit_walltime(&mut self, limit: Option<u64>) -> &mut Self {
        self.limits.walltime = limit;
        self
    }

    pub(crate) fn seccomp(&mut self, seccomp: &Option<Seccomp>) -> &mut Self {
        if let Some(seccomp) = seccomp {
            self.seccomp = Some(Seccomp::new()); // reinitialize
            for syscall in &seccomp.syscalls {
                _ = self._seccomp_allow(syscall);
            }
        } else {
            self.seccomp = None;
        }
        self
    }

    pub fn seccomp_enable(&mut self) -> &mut Self {
        self.seccomp = Some(Seccomp::new());
        self
    }

    pub fn seccomp_allow(&mut self, syscall: &str) -> Result<&mut Self> {
        self._seccomp_allow(syscall)
    }

    fn _seccomp_allow(&mut self, syscall: &str) -> Result<&mut Self> {
        if let Some(seccomp) = &mut self.seccomp {
            ScmpSyscall::from_name(syscall)?;
            seccomp.syscalls.push(syscall.to_string())
        }
        Ok(self)
    }

    pub fn capture_stdout(&mut self, value: bool) -> &mut Self {
        self.capture_stdout = value;
        self
    }

    pub fn capture_stderr(&mut self, value: bool) -> &mut Self {
        self.capture_stderr = value;
        self
    }

    pub fn run(&mut self) -> ExecutorResult {
        match self._run() {
            Ok(val) => val,
            Err(err) => ExecutorResult::failure(&err.to_string()),
        }
    }

    fn _run(&mut self) -> Result<ExecutorResult> {
        let mut out_pipe = contrib::nix::io::pipe()
            .map_err(|err| Error::_ExecutorRunError(format!("create out pipe failed: {}", err)))?;
        let mut err_pipe = contrib::nix::io::pipe()
            .map_err(|err| Error::_ExecutorRunError(format!("create err pipe failed: {}", err)))?;

        _ = unistd::dup2(libc::STDOUT_FILENO, out_pipe.1.as_raw_fd());
        _ = unistd::dup2(libc::STDERR_FILENO, err_pipe.1.as_raw_fd());

        let result = match self.__run(&out_pipe, &err_pipe) {
            Ok(val) => val,
            Err(err) => {
                let err = format!("hakoniwa: {}\n", err);
                _ = unistd::write(err_pipe.1.as_raw_fd(), err.as_bytes());
                _ = unistd::fsync(err_pipe.1.as_raw_fd());
                ExecutorResult::failure(&err.to_string())
            }
        };

        out_pipe.1.close();
        err_pipe.1.close();
        Ok(result)
    }

    fn __run(
        &mut self,
        out_pipe: &contrib::nix::io::Pipe,
        err_pipe: &contrib::nix::io::Pipe,
    ) -> Result<ExecutorResult> {
        self.lookup_executable()?;
        self.log_before_forkexec();

        let _rootfs = contrib::tmpdir::new(&self.rootfs).map_err(|err| {
            Error::_ExecutorRunError(format!("create dir {:?} failed: {}", self.rootfs, err))
        })?;
        let cpr_pipe = contrib::nix::io::pipe().map_err(|err| {
            Error::_ExecutorRunError(format!("create child process result pipe failed: {}", err))
        })?;
        let result = match unsafe { unistd::fork() } {
            Ok(ForkResult::Parent { child, .. }) => self.run_in_parent(child, cpr_pipe),
            Ok(ForkResult::Child) => self.run_in_child(&cpr_pipe, out_pipe, err_pipe),
            Err(err) => ExecutorResult::failure(&format!("fork failed: {}", err)),
        };

        self.log_after_forkexec(&result);
        Ok(result)
    }

    fn run_in_parent(
        &self,
        child: Pid,
        (mut cpr_reader, mut cpr_writer): contrib::nix::io::Pipe,
    ) -> ExecutorResult {
        // Close unused pipe.
        cpr_writer.close();

        // Block until all data is received.
        let result = match ChildProcessResult::recv_from(cpr_reader.as_raw_fd()) {
            Ok(val) => ExecutorResult::from(val),
            Err(err) => ExecutorResult::failure(&format!("recv failed: {}", err)),
        };
        cpr_reader.close();

        // Avoid zombie children.
        _ = signal::kill(child, Signal::SIGKILL);
        _ = wait::waitpid(child, None);

        // .
        result
    }

    fn run_in_child(
        &self,
        cpr_pipe: &contrib::nix::io::Pipe,
        out_pipe: &contrib::nix::io::Pipe,
        err_pipe: &contrib::nix::io::Pipe,
    ) -> ExecutorResult {
        let cpr_pipe = (cpr_pipe.0.as_raw_fd(), cpr_pipe.1.as_raw_fd());
        let out_pipe = (out_pipe.0.as_raw_fd(), out_pipe.1.as_raw_fd());
        let err_pipe = (err_pipe.0.as_raw_fd(), err_pipe.1.as_raw_fd());
        ChildProcess::run(self, cpr_pipe, out_pipe, err_pipe);
        process::exit(0); // unreachable!
    }

    fn lookup_executable(&mut self) -> Result<()> {
        // Absolute? - Assume this is the container path.
        if PathBuf::from(&self.prog).is_absolute() {
            return Ok(());
        }

        // Relative? - Assume the path in the container and in the host are the same.
        if let Some(path) = contrib::pathsearch::find_executable_path(&self.prog) {
            match path.to_str() {
                Some(path) => self.prog = path.to_string(),
                None => todo!(),
            }
            return Ok(());
        }

        // Command not found.
        let err = format!("{}: command not found", self.prog);
        Err(Error::_ExecutorRunError(err))
    }

    fn log_before_forkexec(&self) {
        if !log::log_enabled!(target: "hakoniwa", log::Level::Info) {
            return;
        }

        log::info!(
            "Mount point: host_path: {:?}, container_path: {:?}",
            self.rootfs,
            "/"
        );
        log::info!(
            "Mount point: host_path: none, container_path: {:?}, fstype: proc",
            Mount::PROC_DIR.1,
        );
        if self.mount_new_tmpfs {
            log::info!(
                "Mount point: host_path: none, container_path: {:?}, fstype: tmpfs",
                Mount::TMP_DIR.1,
            );
        }
        if self.mount_new_devfs {
            for path in Mount::NEW_DEVFS_SUBFILES {
                log::info!(
                    "Mount point: host_path: {:?}, container_path: {:?}",
                    path,
                    path,
                );
            }
        }
        for mount in self.mounts.iter() {
            log::info!(
                "Mount point: host_path: {:?}, container_path: {:?}, readonly: {}",
                mount.host_path,
                mount.container_path,
                mount.ms_rdonly_flag().contains(MsFlags::MS_RDONLY)
            );
        }
        if !self.dir.as_os_str().is_empty() {
            log::info!(
                "Mount point: host_path: {:?}, container_path: {:?}",
                self.dir,
                Mount::WORK_DIR.1,
            );
        }

        log::info!(
            "UID map: host_id: {}, container_id: {}",
            self.uid_mappings.host_id,
            self.uid_mappings.container_id,
        );
        log::info!(
            "GID map: host_id: {}, container_id: {}",
            self.gid_mappings.host_id,
            self.gid_mappings.container_id,
        );

        match &self.seccomp {
            Some(seccomp) => {
                log::info!(
                    "Seccomp: enabled (syscalls: {}): {}",
                    seccomp.syscalls.len(),
                    seccomp.syscalls.join(",")
                );
            }
            None => {
                log::info!("Seccomp: disabled");
            }
        }

        log::info!("Execve: {} {:?}", self.prog, self.argv);
    }

    fn log_after_forkexec(&self, result: &ExecutorResult) {
        if !log::log_enabled!(target: "hakoniwa", log::Level::Info) {
            return;
        }

        if let Ok(result) = serde_json::to_string(&result) {
            log::info!("Result: {}", result);
        }
    }
}
