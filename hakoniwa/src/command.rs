use nix::unistd::{self, ForkResult};
use os_pipe::{PipeReader, PipeWriter};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

use crate::{error::*, runc, Child, Container, ExitStatus, Output, Stdio};

/// Process builder, providing fine-grained control over how a new process
/// should be spawned.
///
/// A command is created via [Container::command].
///
/// [Container::command]: crate::Container::command
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
        self.spawn_imp(Stdio::Inherit)
    }

    /// Command#spawn IMP.
    fn spawn_imp(&mut self, default: Stdio) -> Result<Child> {
        let tmpdir = if let Some(dir) = &self.container.rootdir {
            let dir = fs::canonicalize(dir).map_err(ProcessErrorKind::StdIoError)?;
            self.container.rootdir_abspath = dir;
            None
        } else {
            let dir = TempDir::with_prefix("hakoniwa-").map_err(ProcessErrorKind::StdIoError)?;
            self.container.rootdir_abspath = dir.path().to_path_buf();
            Some(dir)
        };

        self.logging();

        let (stdin_reader, stdin_writer) = Self::make_pipe(self.stdin.unwrap_or(default))?;
        let (stdout_reader, stdout_writer) = Self::make_pipe(self.stdout.unwrap_or(default))?;
        let (stderr_reader, stderr_writer) = Self::make_pipe(self.stderr.unwrap_or(default))?;
        let (status_reader, status_writer) = Self::make_pipe(Stdio::MakePipe)?;

        match unsafe { unistd::fork() } {
            Ok(ForkResult::Parent { child, .. }) => {
                drop(stdin_reader);
                drop(stdout_writer);
                drop(stderr_writer);
                drop(status_writer);
                Ok(Child::new(
                    child,
                    stdin_writer,
                    stdout_reader,
                    stderr_reader,
                    status_reader.expect("`status_reader` is used uninitialized"),
                    tmpdir,
                ))
            }
            Ok(ForkResult::Child) => {
                tmpdir.map(|dir| dir.into_path());
                drop(stdin_writer);
                drop(stdout_reader);
                drop(stderr_reader);
                drop(status_reader);
                runc::exec(
                    self,
                    &self.container,
                    stdin_reader,
                    stdout_writer,
                    stderr_writer,
                    status_writer.expect("`status_writer` is used uninitialized"),
                );
                unreachable!();
            }
            Err(err) => Err(ProcessErrorKind::NixError(err))?,
        }
    }

    /// Logging
    fn logging(&self) {
        if !log::log_enabled!(target: "hakoniwa", log::Level::Debug) {
            return;
        }

        let clone_flags = self.container.get_namespaces_clone_flags();
        log::debug!("Unshare namespaces: {:?}", clone_flags);

        log::debug!("RootDir: {:?} -> {:?}", self.container.rootdir_abspath, "/");
        for mount in self.container.get_mounts() {
            log::debug!("Mount: {:?}", mount);
        }

        if let Some(idmap) = &self.container.uidmap {
            log::debug!("UID mapping: {:?}", idmap);
        } else {
            log::debug!("UID mapping: -");
        }
        if let Some(idmap) = &self.container.gidmap {
            log::debug!("GID mapping: {:?}", idmap);
        } else {
            log::debug!("GID mapping: -");
        }

        log::debug!("Execve: {:?}, {:?}", self.program, self.args);
    }

    /// Create a pipe that arranged to connect the parent and child processes.
    fn make_pipe(io: Stdio) -> Result<(Option<PipeReader>, Option<PipeWriter>)> {
        Ok(match io {
            Stdio::Inherit => (None, None),
            Stdio::MakePipe => {
                let pipe = os_pipe::pipe().map_err(ProcessErrorKind::StdIoError)?;
                (Some(pipe.0), Some(pipe.1))
            }
        })
    }

    /// Executes a command as a child process, waiting for it to finish and
    /// collecting its status.
    pub fn status(&mut self) -> Result<ExitStatus> {
        let mut child = self.spawn_imp(Stdio::Inherit)?;
        child.wait()
    }

    /// Executes the command as a child process, waiting for it to finish and
    /// collecting all of its output.
    pub fn output(&mut self) -> Result<Output> {
        let mut child = self.spawn_imp(Stdio::MakePipe)?;
        child.wait_with_output()
    }

    /// Returns the path to the program.
    pub fn get_program(&self) -> &str {
        &self.program
    }

    /// Returns the arguments that will be passed to the program.
    pub fn get_args(&self) -> &Vec<String> {
        &self.args
    }

    /// Returns the environment variables explicitly set for the child process.
    pub fn get_envs(&self) -> &HashMap<String, String> {
        &self.envs
    }

    /// Returns the working directory for the child process.
    pub fn get_current_dir(&self) -> Option<&Path> {
        self.dir.as_deref()
    }
}
