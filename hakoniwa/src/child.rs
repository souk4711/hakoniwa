use nix::sys::signal::{self, Signal};
use nix::sys::wait;
use nix::unistd::Pid;
use os_pipe::{PipeReader, PipeWriter};
use serde::{Deserialize, Serialize};
use std::io::prelude::*;
use std::time::Duration;

use crate::error::*;

/// Information about resource usage.
#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
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
#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
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
    /// The status of the child process.
    pub status: ExitStatus,

    /// The data that the internal process wrote to stdout.
    pub stdout: Vec<u8>,

    /// The data that the internal process wrote to stderr.
    pub stderr: Vec<u8>,
}

/// Representation of a running or exited child process.
pub struct Child {
    pid: Pid,
    status: Option<ExitStatus>,
    status_reader: Option<PipeReader>,
    pub stdin: Option<PipeWriter>,
    pub stdout: Option<PipeReader>,
    pub stderr: Option<PipeReader>,
}

impl Child {
    /// Constructor.
    pub(crate) fn new(
        pid: Pid,
        stdin: Option<PipeWriter>,
        stdout: Option<PipeReader>,
        stderr: Option<PipeReader>,
        status_reader: PipeReader,
    ) -> Self {
        Self {
            pid,
            stdin,
            stdout,
            stderr,
            status: None,
            status_reader: Some(status_reader),
        }
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
    ///
    /// The stdin handle to the child process, if any, will be closed before
    /// waiting. This helps avoid deadlock: it ensures that the child does not
    /// block waiting for input from the parent, while the parent waits for
    /// the child to exit.
    pub fn wait(&mut self) -> Result<ExitStatus> {
        drop(self.stdin.take());
        _ = wait::waitpid(self.pid, None);

        if let Some(mut reader) = self.status_reader.take() {
            let mut encoded = vec![];
            reader
                .read_to_end(&mut encoded)
                .map_err(ProcessErrorKind::StdIoError)?;
            drop(reader);

            let config = bincode::config::standard();
            let (status, _) = bincode::serde::decode_from_slice(&encoded[..], config)
                .map_err(ProcessErrorKind::BincodeDecodeError)?;
            self.status = Some(status);
        }

        let status = self.status.ok_or(ProcessErrorKind::ChildExitStatusGone)?;
        Ok(status)
    }

    /// Simultaneously waits for the child to exit and collect all remaining
    /// output on the stdout/stderr handles, returning an `Output` instance.
    ///
    /// The stdin handle to the child process, if any, will be closed before
    /// waiting. This helps avoid deadlock: it ensures that the child does not
    /// block waiting for input from the parent, while the parent waits for
    /// the child to exit.
    pub fn wait_with_output(&mut self) -> Result<Output> {
        drop(self.stdin.take());

        let (mut stdout, mut stderr) = (vec![], vec![]);
        if let Some(mut reader) = self.stdout.take() {
            reader
                .read_to_end(&mut stdout)
                .map_err(ProcessErrorKind::StdIoError)?;
            drop(reader)
        }
        if let Some(mut reader) = self.stderr.take() {
            reader
                .read_to_end(&mut stderr)
                .map_err(ProcessErrorKind::StdIoError)?;
            drop(reader)
        }

        let status = self.wait()?;
        Ok(Output {
            status,
            stdout,
            stderr,
        })
    }
}
