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
    os::unix::io::RawFd,
    path::{Path, PathBuf},
    process,
    thread::{self, JoinHandle},
    time::Duration,
};

use crate::{
    child_process::{self as ChildProcess, result::ChildProcessResult},
    contrib, Error, IDMap, Limits, Mount, MountType, Namespaces, Result, Seccomp, Stdio, StdioType,
};

/// Result status code.
#[derive(Serialize, Deserialize, PartialEq, Eq, Default, Debug)]
pub enum ExecutorResultStatus {
    /// a.k.a. UK
    ///
    /// Initial value.
    #[default]
    #[serde(rename = "UK")]
    Unknown,

    /// a.k.a. OK
    ///
    /// COMMAND invoked and exited normally, whether success or failure.
    #[serde(rename = "OK")]
    Ok,

    /// a.k.a. SE
    ///
    /// Unexpected error happened.
    #[serde(rename = "SE")]
    SandboxSetupError,

    /// a.k.a. SIG
    ///
    /// Exit with a signal.
    #[serde(rename = "SIG")]
    Signaled,

    /// a.k.a. RFE
    ///
    /// Exit with a special signal - SIGSYS.
    ///
    /// Caused when use [seccomp](Executor::seccomp_enable()) feature.
    #[serde(rename = "RFE")]
    RestrictedFunction,

    /// a.k.a. TLE
    ///
    /// Exit with a special signal - SIGKILL/SIGXCPU.
    ///
    /// Caused when use [limit-cpu][Executor::limit_cpu()] or [limit-walltime](Executor::limit_walltime()) feature.
    #[serde(rename = "TLE")]
    TimeLimitExceeded,

    /// a.k.a. OLE
    ///
    /// Exit with a special signal - SIGXFSZ.
    ///
    /// Caused when use [limit-as](Executor::limit_as()) feature.
    #[serde(rename = "OLE")]
    OutputLimitExceeded,
}

/// Executor execution result.
#[derive(Serialize, Default, Debug)]
pub struct ExecutorResult {
    /// Status code.
    pub status: ExecutorResultStatus,

    /// More info about the status.
    pub reason: String,

    /// Exit code or signal number that caused an exit.
    pub exit_code: Option<i32>,

    /// When process started.
    pub start_time: Option<DateTime<Utc>>,

    /// Wall time used.
    pub real_time: Option<Duration>,

    /// System CPU time used.
    pub system_time: Option<Duration>,

    /// User CPU time used.
    pub user_time: Option<Duration>,

    /// Maximum resident set size (in kilobytes).
    pub max_rss: Option<i64>,

    /// Stdout, only available when use [Executor::stdout()] with [Stdio::initial()].
    #[serde(skip)]
    pub stdout: Vec<u8>,

    /// Stderr, only available when use [Executor::stderr()] with [Stdio::initial()].
    #[serde(skip)]
    pub stderr: Vec<u8>,
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
            ..Default::default()
        }
    }
}

/// **Create and run a new COMMAND which will be executed in a container.**
//// ```
#[derive(Default, Debug)]
pub struct Executor {
    /// The path of the command to run.
    pub(crate) prog: String,

    /// Holds command line arguments.
    pub(crate) argv: Vec<String>,

    /// Holds env variables.
    pub(crate) envp: HashMap<String, String>,

    /// The working directory in container.
    pub(crate) dir: PathBuf,

    /// The rootfs in the host.
    pub(crate) rootfs: PathBuf,

    /// Linux namespaces.
    pub(crate) namespaces: Namespaces,

    /// Process resource limits.
    pub(crate) limits: Limits,

    /// Secure computing.
    pub(crate) seccomp: Option<Seccomp>,

    /// User ID mappings for user namespace.
    pub(crate) uid_mappings: IDMap,

    /// Group ID mappings for user namespace.
    pub(crate) gid_mappings: IDMap,

    /// Hostname for uts namespace.
    pub(crate) hostname: String,

    /// Mount a new tmpfs under "/tmp".
    pub(crate) mount_new_tmpfs: bool,

    /// Mount a new devfs under "/dev".
    pub(crate) mount_new_devfs: bool,

    /// Bind mounts for mount namespace.
    pub(crate) mounts: Vec<Mount>,

    /// Where the stdout write to.
    stdout: Stdio,

    /// Where the stderr write to.
    stderr: Stdio,

    /// Where the stdin read from.
    stdin: Stdio,
}

impl Executor {
    /// This [exit_code][ExecutorResult::exit_code] used when [SandboxSetupError](ExecutorResultStatus::SandboxSetupError).
    pub const EXITCODE_FAILURE: i32 = 125;

    /// Constructor.
    pub fn new<SA: AsRef<str>>(prog: &str, argv: &[SA]) -> Self {
        let uid = Uid::current().as_raw();
        let gid = Gid::current().as_raw();
        Self {
            prog: prog.to_string(),
            argv: argv.iter().map(|arg| String::from(arg.as_ref())).collect(),
            dir: PathBuf::from("/"),
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

    /// Change directory to `dir` in the container.
    pub fn current_dir<P: AsRef<Path>>(&mut self, dir: P) -> Result<&mut Self> {
        if dir.as_ref().is_absolute() {
            self.dir = dir.as_ref().to_path_buf();
            Ok(self)
        } else {
            let err = String::from("should start with a /");
            Err(Error::PathError(dir.as_ref().to_path_buf(), err))
        }
    }

    /// Retain the NETWORK namespace.
    pub fn share_net_ns(&mut self, value: bool) -> &mut Self {
        self.namespaces.net = Some(!value);
        self
    }

    /// Retain the UTS namespace.
    pub fn share_uts_ns(&mut self, value: bool) -> &mut Self {
        self.namespaces.uts = Some(!value);
        self
    }

    /// Set UID to `id` in the container.
    pub fn uid(&mut self, id: u32) -> &mut Self {
        self.uid_mappings.container_id = id;
        self
    }

    /// Set GID to `id` in the container.
    pub fn gid(&mut self, id: u32) -> &mut Self {
        self.gid_mappings.container_id = id;
        self
    }

    /// Set HOSTNAME to `hostname` in the container.
    pub fn hostname(&mut self, hostname: &str) -> &mut Self {
        self.hostname = hostname.to_string();
        self
    }

    /// Mount a new tmpfs under "/tmp" in the container.
    pub fn mount_new_tmpfs(&mut self, mount_new_tmpfs: bool) -> &mut Self {
        self.mount_new_tmpfs = mount_new_tmpfs;
        self
    }

    /// Mount a new devfs under "/dev" in the container.
    ///
    /// Subfiles "/dev/null", "/dev/random", "/dev/urandom", "/dev/zero" will bind mounted.
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

    /// Bind mount the `src` on `dest` with **read-only** access in the container.
    pub fn ro_bind<P1: AsRef<Path>, P2: AsRef<Path>>(
        &mut self,
        src: P1,
        dest: P2,
    ) -> Result<&mut Self> {
        self._bind(src, dest, MountType::RoBind)
    }

    /// Bind mount the `src` on `dest` with **read-write** access in the container.
    pub fn rw_bind<P1: AsRef<Path>, P2: AsRef<Path>>(
        &mut self,
        src: P1,
        dest: P2,
    ) -> Result<&mut Self> {
        self._bind(src, dest, MountType::RwBind)
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

    /// Set an environment variable in the container.
    pub fn setenv(&mut self, name: &str, value: &str) -> &mut Self {
        self.envp.insert(name.to_string(), value.to_string());
        self
    }

    pub(crate) fn limits(&mut self, limits: &Limits) -> &mut Self {
        self.limits = limits.clone();
        self
    }

    /// Limit the maximum size of the COMMAND's virtual memory.
    pub fn limit_as(&mut self, limit: Option<u64>) -> &mut Self {
        self.limits.r#as = limit;
        self
    }

    /// Limit the maximum size of a core file in bytes that the COMMAND may dump.
    pub fn limit_core(&mut self, limit: Option<u64>) -> &mut Self {
        self.limits.core = limit;
        self
    }

    /// Limit the amount of CPU time that the COMMAND can consume, in seconds.
    pub fn limit_cpu(&mut self, limit: Option<u64>) -> &mut Self {
        self.limits.cpu = limit;
        self
    }

    /// Limit the maximum size in bytes of files that the COMMAND may create.
    pub fn limit_fsize(&mut self, limit: Option<u64>) -> &mut Self {
        self.limits.fsize = limit;
        self
    }

    /// Limit the maximum file descriptor number that can be opened by the COMMAND.
    pub fn limit_nofile(&mut self, limit: Option<u64>) -> &mut Self {
        self.limits.nofile = limit;
        self
    }

    /// Limit the amount of wall time that the COMMAND can consume, in seconds.
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

    /// Enable seccomp feature, will use a allowlist to filter syscall.
    pub fn seccomp_enable(&mut self) -> &mut Self {
        self.seccomp = Some(Seccomp::new());
        self
    }

    /// Add a syscall to the allowlist.
    ///
    /// Note that this method should called after [Executor::seccomp_enable()].
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

    /// Where the stdout write to. Default to [Stdio::initial()].
    ///
    /// [Stdio::initial()] - Redirect to [ExecutorResult::stdout].
    ///
    /// [Stdio::inherit()] - Inherit the current process's stdout.
    pub fn stdout(&mut self, io: Stdio) -> &mut Self {
        let io = match io.r#type {
            StdioType::Inherit => Stdio::inherit_stdout(),
            _ => io,
        };
        self.stdout = io;
        self
    }

    /// Where the stderr write to. Default to [Stdio::initial()].
    ///
    /// [Stdio::initial()] - Redirect to [ExecutorResult::stderr].
    ///
    /// [Stdio::inherit()] - Inherit the current process's stderr.
    pub fn stderr(&mut self, io: Stdio) -> &mut Self {
        let io = match io.r#type {
            StdioType::Inherit => Stdio::inherit_stderr(),
            _ => io,
        };
        self.stderr = io;
        self
    }

    /// Where the stdin read from. Default to [Stdio::initial()].
    ///
    /// [Stdio::initial()] - Read nothing.
    ///
    /// [Stdio::inherit()] - Inherit the current process's stdin.
    ///
    /// [Stdio::from::<&str>] - Read bytes from str. Note that currently only support a
    /// str with length less than the pipe's buffer size.
    ///
    /// # Examples
    ///
    /// ```no_run,ignore
    ///     let mut executor = sandbox().command("cat", &["cat"]);
    ///     let result = executor.stdin(Stdio::from("Hako!")).run();
    ///     assert_eq!(String::from_utf8_lossy(&result.stdout), "Hako!");
    /// ```
    pub fn stdin(&mut self, io: Stdio) -> &mut Self {
        let io = match io.r#type {
            StdioType::Inherit => Stdio::inherit_stdin(),
            _ => io,
        };
        self.stdin = io;
        self
    }

    /// Run it in a container.
    pub fn run(&mut self) -> ExecutorResult {
        match self._run() {
            Ok(val) => val,
            Err(err) => ExecutorResult::failure(&err.to_string()),
        }
    }

    fn _run(&mut self) -> Result<ExecutorResult> {
        // Create pipe.
        let mut out_pipe = contrib::nix::io::pipe().map_err(|err| {
            Error::_ExecutorRunError(format!("create stdout pipe failed: {}", err))
        })?;
        let mut err_pipe = contrib::nix::io::pipe().map_err(|err| {
            Error::_ExecutorRunError(format!("create stderr pipe failed: {}", err))
        })?;
        let in_pipe = contrib::nix::io::pipe().map_err(|err| {
            Error::_ExecutorRunError(format!("create stdin pipe failed: {}", err))
        })?;

        // Read stdout/stderr async.
        let out_thr = Self::stream_reader(
            (out_pipe.0.as_raw_fd(), out_pipe.1.as_raw_fd()),
            &self.stdout,
        )?;
        let err_thr = Self::stream_reader(
            (err_pipe.0.as_raw_fd(), err_pipe.1.as_raw_fd()),
            &self.stderr,
        )?;

        // Write stdin.
        Self::stream_writer((in_pipe.0.as_raw_fd(), in_pipe.1.as_raw_fd()), &self.stdin)?;

        // Run & wait.
        let mut result = match self.__run(&out_pipe, &err_pipe, in_pipe) {
            Ok(val) => val,
            Err(err) => {
                let err = format!("hakoniwa: {}\n", err);
                _ = unistd::write(err_pipe.1.as_raw_fd(), err.as_bytes());
                _ = unistd::fsync(err_pipe.1.as_raw_fd());
                ExecutorResult::failure(&err)
            }
        };
        out_pipe.1.close();
        err_pipe.1.close();

        // Wait stdout/stderr.
        if let Some(out_thr) = out_thr {
            result.stdout = out_thr
                .join()
                .map_err(|_| Error::_ExecutorRunError("get stdout data failed".to_string()))?;
        }
        if let Some(err_thr) = err_thr {
            result.stderr = err_thr
                .join()
                .map_err(|_| Error::_ExecutorRunError("get stderr data failed".to_string()))?;
        }

        // Get result.
        Ok(result)
    }

    fn __run(
        &mut self,
        out_pipe: &contrib::nix::io::Pipe,
        err_pipe: &contrib::nix::io::Pipe,
        in_pipe: contrib::nix::io::Pipe,
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
            Ok(ForkResult::Parent { child, .. }) => self.run_in_parent(child, cpr_pipe, in_pipe),
            Ok(ForkResult::Child) => self.run_in_child(&cpr_pipe, out_pipe, err_pipe, &in_pipe),
            Err(err) => ExecutorResult::failure(&format!("fork failed: {}", err)),
        };

        self.log_after_forkexec(&result);
        Ok(result)
    }

    fn run_in_parent(
        &self,
        child: Pid,
        (mut cpr_reader, mut cpr_writer): contrib::nix::io::Pipe,
        (mut in_reader, mut in_writer): contrib::nix::io::Pipe,
    ) -> ExecutorResult {
        // Close unused pipe.
        cpr_writer.close();
        in_reader.close();

        // Stdin.
        in_writer.close();

        // Block until all data is received.
        let result = match ChildProcessResult::recv_from(cpr_reader.as_raw_fd()) {
            Ok(val) => ExecutorResult::from(val),
            Err(err) => ExecutorResult::failure(&format!("recv failed: {}", err)),
        };
        cpr_reader.close();

        // Avoid zombie children.
        _ = signal::kill(child, Signal::SIGKILL);
        _ = wait::waitpid(child, None);

        // Get result.
        result
    }

    fn run_in_child(
        &self,
        cpr_pipe: &contrib::nix::io::Pipe,
        out_pipe: &contrib::nix::io::Pipe,
        err_pipe: &contrib::nix::io::Pipe,
        in_pipe: &contrib::nix::io::Pipe,
    ) -> ExecutorResult {
        let cpr_pipe = (cpr_pipe.0.as_raw_fd(), cpr_pipe.1.as_raw_fd());
        let out_pipe = (out_pipe.0.as_raw_fd(), out_pipe.1.as_raw_fd());
        let err_pipe = (err_pipe.0.as_raw_fd(), err_pipe.1.as_raw_fd());
        let in_pipe = (in_pipe.0.as_raw_fd(), in_pipe.1.as_raw_fd());
        ChildProcess::run(self, cpr_pipe, out_pipe, err_pipe, in_pipe);
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

                log::info!(
                    "Seccomp: use \
                        `sudo ausearch -ts {} -m seccomp -i` \
                            to know more detail",
                    Local::now().format("%H:%M:%S").to_string()
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

    fn stream_reader(pipe: (RawFd, RawFd), io: &Stdio) -> Result<Option<JoinHandle<Vec<u8>>>> {
        match io.r#type {
            StdioType::Initial => Ok(Some(thread::spawn(move || {
                let mut out: Vec<u8> = vec![];
                let mut buf: [u8; 1024] = [0; 1024];
                while let Ok(len) = unistd::read(pipe.0, &mut buf) {
                    match len {
                        0 => break,
                        _ => out.extend_from_slice(&buf[..len]),
                    }
                }
                out
            }))),
            StdioType::Inherit => unistd::dup2(io.as_raw_fd(), pipe.1)
                .map_err(|err| Error::_ExecutorRunError(format!("dup2 failed: {}", err)))
                .map(|_| None::<JoinHandle<Vec<u8>>>),
            StdioType::ByteVector => unreachable!(),
        }
    }

    fn stream_writer(pipe: (RawFd, RawFd), io: &Stdio) -> Result<()> {
        match io.r#type {
            StdioType::Initial => Ok(()),
            StdioType::Inherit => unistd::dup2(io.as_raw_fd(), pipe.0)
                .map_err(|err| Error::_ExecutorRunError(format!("dup2 failed: {}", err)))
                .map(|_| ()),
            StdioType::ByteVector => {
                // Assume this is a small write that will not fill the pipe buffer, so it will
                // not block current thread, otherwise we need a thread::spawn.
                let mut buf = io.as_bytes();
                while !buf.is_empty() {
                    match unistd::write(pipe.1, buf) {
                        Ok(0) => return Ok(()),
                        Ok(n) => buf = &buf[n..],
                        Err(nix::errno::Errno::EINTR) => continue, // interrupted
                        Err(e) => {
                            return Err(Error::_ExecutorRunError(format!("write failed: {}", e)))
                        }
                    }
                }
                Ok(())
            }
        }
    }
}
