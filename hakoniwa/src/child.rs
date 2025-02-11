use nix::sys::signal::{self, Signal};
use nix::sys::wait;
use nix::unistd::Pid;
use serde::{Deserialize, Serialize};
use std::io::prelude::*;
use std::time::Duration;

use crate::error::*;

/// Information about resource usage.
#[derive(Serialize, Deserialize, Debug)]
pub struct Rusage {
    /// Wall clock time.
    pub real_time: Duration,

    /// Total amount of time spent executing in user mode.
    pub user_time: Duration,

    /// Total amount of time spent executing in kernel mode.
    pub system_time: Duration,

    /// The resident set size at its peak, in kilobytes.
    pub max_rss: i64,
}

/// Result of a process after it has terminated.
#[derive(Serialize, Deserialize, Debug)]
pub struct ExitStatus {
    /// The exit code of the child process.
    pub code: i32,

    /// The exit code of the internal process.
    pub exit_code: Option<i32>,

    /// Information about resource usage of the internal process.
    pub rusage: Option<Rusage>,
}

impl ExitStatus {
    pub(crate) const SUCCESS: i32 = 0;
    pub(crate) const FAILURE: i32 = 125;

    /// Was termination successful? Signal termination is not considered a
    /// success, and success is defined as a zero exit status of child process.
    ///
    /// Note the [exit_code][ExitStatus::exit_code] of the internal process
    /// may non-zero.
    pub fn success(&self) -> bool {
        self.code == Self::SUCCESS
    }
}

/// The output of a finished process.
pub struct Output {
    /// The status of the process.
    pub status: ExitStatus,

    /// The data that the process wrote to stdout.
    pub stdout: Vec<u8>,

    /// The data that the process wrote to stderr.
    pub stderr: Vec<u8>,
}

/// Representation of a running or exited child process.
pub struct Child {
    pid: Pid,
    reader: os_pipe::PipeReader,
}

impl Child {
    /// Constructor.
    pub(crate) fn new(pid: Pid, reader: os_pipe::PipeReader) -> Self {
        Self { pid, reader }
    }

    /// Returns the OS-assigned process identifier associated with this child.
    pub fn id(&self) -> u32 {
        self.pid.as_raw() as u32
    }

    /// Forces the child process to exit.
    pub fn kill(&mut self) -> Result<()> {
        _ = signal::kill(self.pid, Signal::SIGKILL);
        Ok(())
    }

    /// Waits for the child to exit completely, returning the status that it
    /// exited with.
    pub fn wait(&mut self) -> Result<ExitStatus> {
        _ = wait::waitpid(self.pid, None);

        let mut encoded: [u8; 1024] = [0; 1024];
        self.reader
            .read(&mut encoded)
            .map_err(ProcessErrorKind::StdIoError)?;

        let config = bincode::config::standard();
        let (status, _) = bincode::serde::decode_from_slice(&encoded[..], config)
            .map_err(ProcessErrorKind::BincodeDecodeError)?;

        Ok(status)
    }

    /// Simultaneously waits for the child to exit and collect all remaining
    /// output on the stdout/stderr handles, returning an `Output` instance.
    pub fn wait_with_output(&mut self) -> Result<Output> {
        let status = self.wait()?;
        let stdout = vec![];
        let stderr = vec![];
        Ok(Output {
            status,
            stdout,
            stderr,
        })
    }
}
