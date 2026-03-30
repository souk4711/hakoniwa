use nix::sys::signal::{self, Signal};
use nix::unistd::{self, ForkResult, Pid};
use std::collections::HashMap;
use std::fs;
use std::io::prelude::*;
use std::io::{PipeReader, PipeWriter, pipe};
use std::path::{Path, PathBuf};
use tempfile::TempDir;

use crate::{Child, Container, ExitStatus, Namespace, Output, Stdio, error::*};

/// Process builder, providing fine-grained control over how a new process
/// should be spawned.
///
/// A command is created via [Container::command]. This struct is similar
/// to [std::process::Command].
///
/// [Container::command]: crate::Container::command
/// [std::process::Command]: https://doc.rust-lang.org/std/process/struct.Command.html
pub struct Command {
    container: Container,
    program: String,
    args: Vec<String>,
    envs: HashMap<String, String>,
    dir: Option<PathBuf>,
    stdin: Option<Stdio>,
    stdout: Option<Stdio>,
    stderr: Option<Stdio>,
    pub(crate) wait_timeout: Option<u64>,
    #[cfg(feature = "cgroups")]
    pub(crate) running_cgroup: Option<crate::cgroups::Manager>,
    pub(crate) running_rootdir_abspath: PathBuf,
}

impl Command {
    /// Constructs a new Command for launching the program at path `program`
    /// within `container`.
    pub(crate) fn new(program: &str, container: Container) -> Self {
        Self {
            container,
            program: program.to_string(),
            args: vec![],
            envs: HashMap::new(),
            dir: None,
            stdin: None,
            stdout: None,
            stderr: None,
            wait_timeout: None,
            #[cfg(feature = "cgroups")]
            running_cgroup: None,
            running_rootdir_abspath: PathBuf::new(),
        }
    }

    /// Adds an argument to pass to the program.
    pub fn arg(&mut self, arg: &str) -> &mut Self {
        self.args.push(arg.to_string());
        self
    }

    /// Adds multiple arguments to pass to the program.
    pub fn args<I, S>(&mut self, args: I) -> &mut Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        for arg in args {
            self.arg(arg.as_ref());
        }
        self
    }

    /// Inserts or updates an explicit environment variable mapping.
    pub fn env(&mut self, key: &str, val: &str) -> &mut Self {
        self.envs.insert(key.to_string(), val.to_string());
        self
    }

    /// Inserts or updates multiple explicit environment variable mappings.
    pub fn envs<I, K, V>(&mut self, vars: I) -> &mut Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: AsRef<str>,
        V: AsRef<str>,
    {
        for (key, val) in vars {
            self.env(key.as_ref(), val.as_ref());
        }
        self
    }

    /// Sets the working directory for the child process.
    pub fn current_dir<P: AsRef<Path>>(&mut self, dir: P) -> &mut Self {
        self.dir = Some(dir.as_ref().to_path_buf());
        self
    }

    /// Sets the number of seconds to wait for the child process to terminate.
    pub fn wait_timeout(&mut self, timeout: u64) -> &mut Self {
        self.wait_timeout = Some(timeout);
        self
    }

    /// Configuration for the child process’s standard input (stdin) handle.
    ///
    /// Defaults to [inherit] when used with [spawn] or [status], and defaults
    /// to [piped] when used with [output].
    ///
    /// [inherit]: Stdio::inherit
    /// [piped]: Stdio::piped
    /// [spawn]: Command::spawn
    /// [status]: Command::status
    /// [output]: Command::output
    pub fn stdin(&mut self, cfg: Stdio) -> &mut Self {
        self.stdin = Some(cfg);
        self
    }

    /// Configuration for the child process’s standard output (stdout) handle.
    ///
    /// Defaults to [inherit] when used with [spawn] or [status], and defaults
    /// to [piped] when used with [output].
    ///
    /// [inherit]: Stdio::inherit
    /// [piped]: Stdio::piped
    /// [spawn]: Command::spawn
    /// [status]: Command::status
    /// [output]: Command::output
    pub fn stdout(&mut self, cfg: Stdio) -> &mut Self {
        self.stdout = Some(cfg);
        self
    }

    /// Configuration for the child process’s standard error (stderr) handle.
    ///
    /// Defaults to [inherit] when used with [spawn] or [status], and defaults
    /// to [piped] when used with [output].
    ///
    /// [inherit]: Stdio::inherit
    /// [piped]: Stdio::piped
    /// [spawn]: Command::spawn
    /// [status]: Command::status
    /// [output]: Command::output
    pub fn stderr(&mut self, cfg: Stdio) -> &mut Self {
        self.stderr = Some(cfg);
        self
    }

    /// Executes the command as a child process, returning a handle to it.
    pub fn spawn(&mut self) -> Result<Child> {
        self.spawn_imp(true)
    }

    /// Command#spawn IMP.
    fn spawn_imp(&mut self, default_inherit: bool) -> Result<Child> {
        let tmpdir = if let Some(dir) = &self.container.rootdir {
            let dir = fs::canonicalize(dir).map_err(ProcessErrorKind::StdIoError)?;
            self.running_rootdir_abspath = dir;
            None
        } else {
            let dir = TempDir::with_prefix("hakoniwa-").map_err(ProcessErrorKind::StdIoError)?;
            self.running_rootdir_abspath = dir.path().to_path_buf();
            Some(dir)
        };

        self.logging();

        let default_stdio = || {
            if default_inherit {
                Stdio::Inherit
            } else {
                Stdio::MakePipe
            }
        };
        let (stdin_reader, stdin_writer) =
            Stdio::into_ends(self.stdin.take().unwrap_or(default_stdio()), false)?;
        let (stdout_reader, stdout_writer) =
            Stdio::into_ends(self.stdout.take().unwrap_or(default_stdio()), true)?;
        let (stderr_reader, stderr_writer) =
            Stdio::into_ends(self.stderr.take().unwrap_or(default_stdio()), true)?;
        let mut pipe_a = pipe().map_err(ProcessErrorKind::StdIoError)?;
        let mut pipe_z = pipe().map_err(ProcessErrorKind::StdIoError)?;

        match unsafe { unistd::fork() } {
            Ok(ForkResult::Parent { child, .. }) => {
                drop(stdin_reader);
                drop(stdout_writer);
                drop(stderr_writer);
                drop(pipe_a.1);
                drop(pipe_z.0);

                let mut noleading = false;
                let mut status = None;
                let r = self.mainp_setup(&mut pipe_a.0, &mut pipe_z.1, child);
                match r {
                    // All goes well.
                    Ok(0) => {
                        // ;
                    }
                    // The child process exited early due to some errors.
                    Ok(1) => {
                        noleading = true;
                    }
                    // Unreachable.
                    Ok(_) => {
                        unreachable!("Command::spawn_imp");
                    }
                    // The main process setup failed due to:
                    //
                    //  - SetupUGidmapFailed: uid range not allowed, etc.
                    //  - SetupNetworkFailed: command pasta not found, etc.
                    //  - StdIoError: failed to write to pipe cuz the child process force killed by taskmgr.
                    Err(e) => {
                        _ = signal::kill(child, Signal::SIGKILL);
                        status = Some(ExitStatus::new_failure(&e.to_string()));
                    }
                };

                drop(pipe_z.1);
                Ok(Child::new(
                    child,
                    stdin_writer.and_then(|w| w.into_pipe_writer()),
                    stdout_reader.and_then(|w| w.into_pipe_reader()),
                    stderr_reader.and_then(|w| w.into_pipe_reader()),
                    pipe_a.0,
                    noleading,
                    status,
                    tmpdir,
                    #[cfg(feature = "cgroups")]
                    self.running_cgroup.take(),
                ))
            }
            Ok(ForkResult::Child) => {
                tmpdir.map(|dir| dir.keep());
                drop(stdin_writer);
                drop(stdout_reader);
                drop(stderr_reader);
                drop(pipe_a.0);
                drop(pipe_z.1);
                crate::runc::exec(
                    self,
                    &self.container,
                    stdin_reader,
                    stdout_writer,
                    stderr_writer,
                    pipe_z.0,
                    pipe_a.1,
                );
                unreachable!("Command::spawn_imp");
            }
            Err(err) => Err(ProcessErrorKind::NixError(err))?,
        }
    }

    /// Logging.
    fn logging(&self) {
        if !log::log_enabled!(log::Level::Debug) {
            return;
        }

        let namespaces = self.container.namespaces.clone();
        if namespaces.is_empty() {
            log::debug!("Unshare namespaces: NULL");
        } else {
            let mut namespaces = namespaces
                .into_iter()
                .map(|k| k.to_string())
                .collect::<Vec<_>>();
            namespaces.sort();
            log::debug!("Unshare namespaces: {}", namespaces.join(", "));
        }

        if self.container.namespaces.contains(&Namespace::Mount) {
            log::debug!(
                "Mount:    root: {}",
                self.running_rootdir_abspath.to_string_lossy(),
            );
            for mount in self.container.get_mounts() {
                log::debug!("Mount: {mount}");
            }
            for op in self.container.get_fs_operations() {
                log::debug!("FsOperation: {op}");
            }
        }

        if self.container.namespaces.contains(&Namespace::User) {
            if let Some(idmaps) = &self.container.uidmaps {
                for idmap in idmaps {
                    log::debug!("UID mapping: {idmap}");
                }
            } else {
                log::debug!("UID mapping: -");
            }
            if let Some(idmaps) = &self.container.gidmaps {
                for idmap in idmaps {
                    log::debug!("GID mapping: {idmap}");
                }
            } else {
                log::debug!("GID mapping: -");
            }
        }

        for (k, v) in self.get_envs() {
            log::debug!("Env: {k}={v}")
        }

        #[cfg(feature = "landlock")]
        if let Some(ruleset) = &self.container.landlock_ruleset {
            use crate::landlock::*;

            if !ruleset.restrictions.is_empty() {
                let mut resources = ruleset
                    .restrictions
                    .keys()
                    .map(|k| k.to_string())
                    .collect::<Vec<_>>();
                resources.sort();
                log::debug!("Landlock: {}", resources.join(", "));
            }

            if ruleset.restrictions.contains_key(&Resource::FS) {
                for rule in &ruleset.get_fs_rules() {
                    log::trace!("Landlock FS rule: {rule}");
                }
            }

            for resource in [Resource::NET_TCP_BIND, Resource::NET_TCP_CONNECT] {
                if !ruleset.restrictions.contains_key(&resource) {
                    continue;
                }
                let rules = match ruleset.net_rules.get(&resource) {
                    Some(rules) => rules,
                    None => continue,
                };
                for rule in rules {
                    log::trace!("Landlock NET rule: {rule}");
                }
            }
        }

        #[cfg(feature = "seccomp")]
        if let Some(filter) = &self.container.seccomp_filter {
            let mut arches = filter
                .architectures
                .iter()
                .map(|arch| format!("{arch:?}"))
                .collect::<Vec<_>>();
            arches.sort();
            log::debug!(
                "Seccomp: Load {} rules for architectures({})",
                filter.rules.len() + 1,
                arches.join(", ")
            );

            log::trace!("Seccomp rule: ... -> {:?}", filter.default_action);
            for rule in &filter.rules {
                log::trace!("Seccomp rule: {rule}");
            }
        }

        log::debug!("Execve: {:?}, {:?}", self.program, self.args);
        log::debug!("================================");
    }

    /// Setup [ug]idmap, network, cgroups, etc.
    fn mainp_setup(
        &mut self,
        reader: &mut PipeReader,
        writer: &mut PipeWriter,
        child: Pid,
    ) -> Result<u8> {
        loop {
            // Receive the child process's request.
            let mut request = [0];
            reader
                .read_exact(&mut request)
                .map_err(ProcessErrorKind::StdIoError)?;

            // The child process exited early due to some errors, so there is no need to do any setup.
            if request[0] == crate::runc::FIN {
                return Ok(1);
            }

            // Setup completed.
            if request[0] == crate::runc::SETUP_SUCCESS {
                return Ok(0);
            };

            // Setup [ug]idmap.
            if request[0] & crate::runc::SETUP_UGIDMAP == crate::runc::SETUP_UGIDMAP {
                self.mainp_setup_ugidmap(child)?;
            };

            // Setup network.
            if request[0] & crate::runc::SETUP_NETWORK == crate::runc::SETUP_NETWORK {
                self.mainp_setup_network(child)?;
            };

            // Setup cgroups.
            if request[0] & crate::runc::SETUP_CGROUPS == crate::runc::SETUP_CGROUPS {
                #[cfg(feature = "cgroups")]
                self.mainp_setup_cgroups(child)?;
            };

            // Send a response back to the child process.
            writer
                .write_all(&[0])
                .map_err(ProcessErrorKind::StdIoError)?;
        }
    }

    /// Setup [ug]idmap.
    fn mainp_setup_ugidmap(&self, child: Pid) -> Result<()> {
        crate::unshare::mainp_setup_ugidmap(&self.container, child)
    }

    /// Setup network.
    fn mainp_setup_network(&self, child: Pid) -> Result<()> {
        crate::unshare::mainp_setup_network(&self.container, child)
    }

    /// Setup cgroups.
    #[cfg(feature = "cgroups")]
    fn mainp_setup_cgroups(&mut self, child: Pid) -> Result<()> {
        let resources = &self
            .container
            .cgroups_resources
            .clone()
            .expect("Container#cgroups_resources is some");

        let cgroup = crate::cgroups::Manager::new(&format!("{child}"))
            .map_err(ProcessErrorKind::SetupCgroupsFailed)?;
        cgroup
            .apply(child, resources)
            .map_err(ProcessErrorKind::SetupCgroupsFailed)?;

        self.running_cgroup = Some(cgroup);
        Ok(())
    }

    /// Executes a command as a child process, waiting for it to finish and
    /// collecting its status.
    pub fn status(&mut self) -> Result<ExitStatus> {
        let mut child = self.spawn_imp(true)?;
        child.wait()
    }

    /// Executes the command as a child process, waiting for it to finish and
    /// collecting all of its output.
    pub fn output(&mut self) -> Result<Output> {
        let mut child = self.spawn_imp(false)?;
        child.wait_with_output()
    }

    /// Returns the path to the program.
    pub fn get_program(&self) -> &str {
        &self.program
    }

    /// Returns the arguments that will be passed to the program.
    pub fn get_args(&self) -> Vec<String> {
        self.args.clone()
    }

    /// Returns the environment variables explicitly set for the child process.
    pub fn get_envs(&self) -> HashMap<String, String> {
        self.envs.clone()
    }

    /// Returns the working directory for the child process.
    pub fn get_current_dir(&self) -> Option<&Path> {
        self.dir.as_deref()
    }
}
