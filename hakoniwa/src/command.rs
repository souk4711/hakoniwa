use nix::unistd::{self, ForkResult};
use std::collections::HashMap;

use crate::{error::*, runc, Child, Container, ExitStatus, Output};

/// Process builder, providing fine-grained control over how a new process
/// should be spawned.
pub struct Command {
    container: Container,
    program: String,
    args: Vec<String>,
    envs: HashMap<String, String>,
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
            wait_timeout: None,
        }
    }

    /// Adds an argument to pass to the program.
    pub fn arg(&mut self, arg: &str) -> &mut Self {
        self.args.push(arg.to_string());
        self
    }

    /// Adds multiple arguments to pass to the program.
    pub fn args<S: AsRef<str>>(&mut self, args: &[S]) -> &mut Self {
        for arg in args {
            self.arg(arg.as_ref());
        }
        self
    }

    /// Sets the number of seconds to wait for the child process to terminate.
    pub fn wait_timeout(&mut self, timeout: u64) -> &mut Self {
        self.wait_timeout = Some(timeout);
        self
    }

    /// Executes the command as a child process, returning a handle to it.
    pub fn spawn(&mut self) -> Result<Child> {
        let (stdin_reader, stdin_writer) = os_pipe::pipe().map_err(ProcessErrorKind::StdIoError)?;
        let (stdout_reader, stdout_writer) =
            os_pipe::pipe().map_err(ProcessErrorKind::StdIoError)?;
        let (stderr_reader, stderr_writer) =
            os_pipe::pipe().map_err(ProcessErrorKind::StdIoError)?;
        let (status_reader, status_writer) =
            os_pipe::pipe().map_err(ProcessErrorKind::StdIoError)?;

        match unsafe { unistd::fork() } {
            Ok(ForkResult::Parent { child, .. }) => {
                drop(stdin_reader);
                drop(stdout_writer);
                drop(stderr_writer);
                drop(status_writer);
                Ok(Child::new(
                    child,
                    Some(stdin_writer),
                    Some(stdout_reader),
                    Some(stderr_reader),
                    status_reader,
                ))
            }
            Ok(ForkResult::Child) => {
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
                    status_writer,
                );
                unreachable!();
            }
            Err(err) => Err(ProcessErrorKind::NixError(err))?,
        }
    }

    /// Executes a command as a child process, waiting for it to finish and
    /// collecting its status.
    pub fn status(&mut self) -> Result<ExitStatus> {
        let mut child = self.spawn()?;
        child.wait()
    }

    /// Executes the command as a child process, waiting for it to finish and
    /// collecting all of its output.
    pub fn output(&mut self) -> Result<Output> {
        let mut child = self.spawn()?;
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
}
