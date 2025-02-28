use nix::sys::signal::{self, Signal};
use nix::sys::wait;
use nix::unistd::Pid;
use os_pipe::{PipeReader, PipeWriter};
use serde::{Deserialize, Serialize};
use std::io::prelude::*;
use std::thread;
use std::time::Duration;
use std::{fmt, str};
use tempfile::TempDir;

use crate::error::*;

/// Information about resource usage.
#[derive(Serialize, Deserialize, Clone, Debug)]
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
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ExitStatus {
    /// The exit code of the child process.
    pub code: i32,

    /// The detailed message of the [code][ExitStatus::code].
    pub reason: String,

    /// The exit code of the internal process.
    pub exit_code: Option<i32>,

    /// Information about resource usage of the internal process.
    pub rusage: Option<Rusage>,
}

impl ExitStatus {
    pub(crate) const SUCCESS: i32 = 0;
    pub(crate) const FAILURE: i32 = 125; // If the Container itself fails.

    /// Was termination successful? Signal termination is not considered a
    /// success, and success is defined as a zero exit status of internal process.
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

impl fmt::Debug for Output {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        let stdout_utf8 = str::from_utf8(&self.stdout);
        let stdout_debug: &dyn fmt::Debug = match stdout_utf8 {
            Ok(ref str) => str,
            Err(_) => &self.stdout,
        };

        let stderr_utf8 = str::from_utf8(&self.stderr);
        let stderr_debug: &dyn fmt::Debug = match stderr_utf8 {
            Ok(ref str) => str,
            Err(_) => &self.stderr,
        };

        fmt.debug_struct("Output")
            .field("status", &self.status)
            .field("stdout", stdout_debug)
            .field("stderr", stderr_debug)
            .finish()
    }
}

/// Representation of a running or exited child process.
///
/// A child process is created via the [Command::spawn].
///
/// [Command::spawn]: crate::Command::spawn
pub struct Child {
    pid: Pid,
    status: Option<ExitStatus>,
    status_reader: Option<PipeReader>,
    tmpdir: Option<TempDir>,
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
        tmpdir: Option<TempDir>,
    ) -> Self {
        Self {
            pid,
            stdin,
            stdout,
            stderr,
            status: None,
            status_reader: Some(status_reader),
            tmpdir,
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

        self.logging();

        drop(self.tmpdir.take());
        Ok(self
            .status
            .clone()
            .ok_or(ProcessErrorKind::ChildExitStatusGone)?)
    }

    /// Logging.
    fn logging(&self) {
        if !log::log_enabled!(target: "hakoniwa", log::Level::Debug) {
            return;
        }

        if let Some(status) = &self.status {
            log::debug!("Exited: {}", status.reason);

            if let Some(rusage) = &status.rusage {
                log::debug!("Rusage: real time: {:?}", rusage.real_time);
                log::debug!("Rusage: user time: {:?}", rusage.user_time);
                log::debug!("Rusage:  sys time: {:?}", rusage.system_time);
            }
        } else {
            log::debug!("Exited: NULL");
        }
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
        match (self.stdout.take(), self.stderr.take()) {
            (None, None) => {}
            (Some(mut out), None) => {
                out.read_to_end(&mut stdout)
                    .map(drop)
                    .map_err(ProcessErrorKind::StdIoError)?;
            }
            (None, Some(mut err)) => {
                err.read_to_end(&mut stderr)
                    .map(drop)
                    .map_err(ProcessErrorKind::StdIoError)?;
            }
            (Some(mut out), Some(mut err)) => {
                self.read2(&mut out, &mut stdout, &mut err, &mut stderr)?;
            }
        }

        let status = self.wait()?;
        Ok(Output {
            status,
            stdout,
            stderr,
        })
    }

    fn read2(
        &mut self,
        out: &mut PipeReader,
        stdout: &mut Vec<u8>,
        err: &mut PipeReader,
        stderr: &mut Vec<u8>,
    ) -> Result<()> {
        thread::scope(|s| {
            let throut = s.spawn(move || out.read_to_end(stdout).map(drop));
            let threrr = s.spawn(move || err.read_to_end(stderr).map(drop));

            let r = throut.join();
            match r {
                Err(_) => return Err(ProcessErrorKind::StdThreadPanic),
                Ok(Err(r)) => return Err(ProcessErrorKind::StdIoError(r)),
                Ok(Ok(_)) => {}
            }

            let r = threrr.join();
            match r {
                Err(_) => Err(ProcessErrorKind::StdThreadPanic),
                Ok(r) => r.map_err(ProcessErrorKind::StdIoError),
            }
        })?;
        Ok(())
    }
}
