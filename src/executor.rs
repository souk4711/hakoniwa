use chrono::prelude::*;
use nix::{
    sys::signal::{self, Signal},
    sys::wait,
    unistd::{self, ForkResult, Gid, Pid, Uid},
};
use scopeguard::defer;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs,
    os::unix::io::RawFd,
    path::{Path, PathBuf},
    process,
    time::Duration,
};

use crate::{
    child_process::{self as ChildProcess, result::ChildProcessResult},
    contrib, Error, IDMap, Limits, Mount, MountType, Namespaces, Result,
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
    pub fn failure(reason: &str) -> Self {
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
    pub(crate) dir: PathBuf,                  // specifies the working directory of the process
    pub(crate) limits: Limits,
    pub(crate) namespaces: Namespaces,
    pub(crate) uid_mappings: IDMap, // user ID mappings for user namespace
    pub(crate) gid_mappings: IDMap, // group ID mappings for user namespace
    pub(crate) hostname: String,    // hostname for uts namespace
    pub(crate) rootfs: PathBuf,     // rootfs for mount namespace
    pub(crate) mounts: Vec<Mount>,  // bind mounts for mount namespace
}

impl Executor {
    pub(crate) const EXITCODE_FAILURE: i32 = 125;

    pub fn new<SA: AsRef<str>>(prog: &str, argv: &[SA]) -> Self {
        let uid = Uid::current().as_raw();
        let gid = Gid::current().as_raw();
        Self {
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
            rootfs: contrib::fs::temp_dir("hakoniwa"),
            ..Default::default()
        }
    }

    pub fn current_dir<P: AsRef<Path>>(&mut self, dir: P) -> Result<&mut Self> {
        match fs::canonicalize(&dir) {
            Ok(val) => {
                self.dir = val;
                Ok(self)
            }
            Err(err) => {
                let err = err.to_string();
                Err(Error::PathError(dir.as_ref().to_path_buf(), err))
            }
        }
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

    pub fn limit_walltime(&mut self, limit: Option<u64>) -> &mut Self {
        self.limits.walltime = limit;
        self
    }

    pub(crate) fn namespaces(&mut self, namespaces: Namespaces) -> &mut Self {
        self.namespaces = namespaces;
        self
    }

    pub fn mounts(&mut self, mounts: Vec<Mount>) -> &mut Self {
        for mount in mounts {
            _ = self._bind(mount.host_path, mount.container_path, mount.r#type);
        }
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

    pub fn bind<P1: AsRef<Path> + std::fmt::Debug, P2: AsRef<Path>>(
        &mut self,
        src: P1,
        dest: P2,
    ) -> Result<&mut Self> {
        self._bind(src, dest, MountType::Bind)
    }

    pub fn ro_bind<P1: AsRef<Path> + std::fmt::Debug, P2: AsRef<Path>>(
        &mut self,
        src: P1,
        dest: P2,
    ) -> Result<&mut Self> {
        self._bind(src, dest, MountType::RoBind)
    }

    pub fn run(&mut self) -> ExecutorResult {
        self.prog = match Self::find_executable_path(&self.prog) {
            Some(path) => match path.to_str() {
                Some(path) => path.to_string(),
                None => unreachable!(),
            },
            None => {
                let err = format!("{}: command not found", self.prog);
                return Self::failure_result(&err);
            }
        };

        match fs::create_dir(&self.rootfs) {
            Ok(_) => {}
            Err(err) => {
                let err = format!("create dir {:?} failed: {}", self.rootfs, err);
                return Self::failure_result(&err);
            }
        };
        defer! { _ = fs::remove_dir_all(&self.rootfs) }

        let cpr_pipe = match unistd::pipe() {
            Ok(val) => val,
            Err(err) => {
                let err = format!("create cpr pipe failed: {}", err);
                return Self::failure_result(&err);
            }
        };

        match unsafe { unistd::fork() } {
            Ok(ForkResult::Parent { child, .. }) => self.run_in_parent(child, cpr_pipe),
            Ok(ForkResult::Child) => self.run_in_child(cpr_pipe),
            Err(err) => {
                _ = unistd::close(cpr_pipe.0);
                _ = unistd::close(cpr_pipe.1);
                let err = format!("fork failed: {}", err);
                Self::failure_result(&err)
            }
        }
    }

    fn run_in_parent(
        &self,
        child: Pid,
        (cpr_reader, cpr_writer): (RawFd, RawFd),
    ) -> ExecutorResult {
        // Avoid zombie children.
        defer! {
            _ = signal::kill(child, Signal::SIGKILL);
            _ = wait::waitpid(child, None);
        }

        // Close unused pipe.
        _ = unistd::close(cpr_writer);
        defer! { _ = unistd::close(cpr_reader); }

        // Block until all data is received.
        match ChildProcessResult::recv_from(cpr_reader) {
            Ok(val) => ExecutorResult::from(val),
            Err(err) => {
                let err = format!("recv from child process failed: {}", err);
                Self::failure_result(&err)
            }
        }
    }

    fn run_in_child(&self, cpr_pipe: (RawFd, RawFd)) -> ExecutorResult {
        ChildProcess::run(self, cpr_pipe);
        process::exit(0); // unreachable!
    }

    fn failure_result(reason: &str) -> ExecutorResult {
        ExecutorResult::failure(reason)
    }

    fn find_executable_path(prog: &str) -> Option<PathBuf> {
        let path = PathBuf::from(prog);
        if path.is_absolute() {
            // Assume this is the container path.
            Some(path)
        } else {
            // Assume the path in the container and in the host are the same.
            contrib::fs::find_executable_path(prog)
        }
    }

    fn _bind<P1: AsRef<Path> + std::fmt::Debug, P2: AsRef<Path>>(
        &mut self,
        src: P1,
        dest: P2,
        r#type: MountType,
    ) -> Result<&mut Self> {
        match fs::canonicalize(&src) {
            Ok(val) => {
                self.mounts.push(Mount::new(&val, dest, r#type));
                Ok(self)
            }
            Err(err) => {
                let err = err.to_string();
                Err(Error::PathError(src.as_ref().to_path_buf(), err))
            }
        }
    }
}
