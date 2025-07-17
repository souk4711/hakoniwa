use nix::sys::signal::{self, Signal};
use nix::sys::wait::{self, WaitPidFlag, WaitStatus};
use nix::unistd::Pid;
use serde::{Deserialize, Serialize};
use std::io::prelude::*;
use std::io::{PipeReader, PipeWriter};
use std::thread;
use std::{fmt, str};
use tempfile::TempDir;

use crate::{error::*, Command, ProcPidSmapsRollup, ProcPidStatus, Rusage};

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

    /// Accumulated smaps stats for all mappings of the internal process.
    pub proc_pid_smaps_rollup: Option<ProcPidSmapsRollup>,

    /// Memory usage and status information of the internal process.
    pub proc_pid_status: Option<ProcPidStatus>,
}

impl ExitStatus {
    pub(crate) const SUCCESS: i32 = 0;
    pub(crate) const FAILURE: i32 = 125; // If the Container itself fails.

    /// Constructs a new ExitStatus with FAILURE code.
    pub(crate) fn new_failure(reason: &str) -> Self {
        Self {
            code: Self::FAILURE,
            reason: reason.to_string(),
            exit_code: None,
            rusage: None,
            proc_pid_smaps_rollup: None,
            proc_pid_status: None,
        }
    }

    /// Constructs a new ExitStatus from nix::sys::wait::WaitStatus.
    pub(crate) fn from_wait_status(ws: &WaitStatus, command: &Command) -> Self {
        let program = command.get_program();
        match *ws {
            WaitStatus::Exited(_, status) => Self {
                code: status,
                reason: format!("process({program}) exited with code {status}"),
                exit_code: Some(status),
                rusage: None,
                proc_pid_smaps_rollup: None,
                proc_pid_status: None,
            },
            WaitStatus::Signaled(_, signal, _) => Self {
                code: 128 + signal as i32,
                reason: format!("process({program}) received signal {signal}"),
                exit_code: None,
                rusage: None,
                proc_pid_smaps_rollup: None,
                proc_pid_status: None,
            },
            _ => {
                unreachable!();
            }
        }
    }

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
/// A child process is created via the [Command::spawn]. This struct is similar
/// to [std::process::Child].
///
/// [Command::spawn]: crate::Command::spawn
/// [std::process::Child]: https://doc.rust-lang.org/std/process/struct.Child.html
pub struct Child {
    pid: Pid,
    status: Option<ExitStatus>,
    status_reader: Option<PipeReader>,
    status_reader_noleading: bool,
    tmpdir: Option<TempDir>,
    pub stdin: Option<PipeWriter>,
    pub stdout: Option<PipeReader>,
    pub stderr: Option<PipeReader>,
}

impl Child {
    /// Constructor.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        pid: Pid,
        stdin: Option<PipeWriter>,
        stdout: Option<PipeReader>,
        stderr: Option<PipeReader>,
        status_reader: PipeReader,
        status_reader_noleading: bool,
        status: Option<ExitStatus>,
        tmpdir: Option<TempDir>,
    ) -> Self {
        Self {
            pid,
            stdin,
            stdout,
            stderr,
            status_reader: Some(status_reader),
            status_reader_noleading,
            status,
            tmpdir,
        }
    }

    /// Returns the OS-assigned process identifier associated with this child.
    pub fn id(&self) -> u32 {
        self.pid.as_raw() as u32
    }

    /// Forces the child process to exit.
    pub fn kill(&mut self) -> Result<()> {
        // If we've already waited on this process then the pid can be recycled
        // and used for another process, and we probably shouldn't be killing
        // random processes, so return Ok because the process has exited already.
        if self.status.is_some() {
            return Ok(());
        }

        _ = signal::kill(self.pid, Signal::SIGKILL);
        Ok(())
    }

    /// Attempts to collect the exit status of the child if it has already exited.
    ///
    /// This function will not block the calling thread and will only check to see
    /// if the child process has exited or not. If the child has exited then on Unix
    /// the process ID is reaped. This function is guaranteed to repeatedly return
    /// a successful exit status so long as the child has already exited.
    ///
    /// If the child has exited, then Ok(Some(status)) is returned. If the exit
    /// status is not available at this time then Ok(None) is returned. If an error
    /// occurs, then that error is returned.
    ///
    /// Note that unlike wait, this function will not attempt to drop stdin.
    pub fn try_wait(&mut self) -> Result<Option<ExitStatus>> {
        if let Some(status) = &self.status {
            return Ok(Some(status.clone()));
        }

        let flags = Some(WaitPidFlag::WNOHANG);
        let ws = wait::waitpid(self.pid, flags).map_err(ProcessErrorKind::NixError)?;
        if let WaitStatus::StillAlive = ws {
            Ok(None)
        } else {
            Ok(Some(self.retrieve_exit_status(ws)?))
        }
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

        if let Some(status) = &self.status {
            return Ok(status.clone());
        }

        let flags = None;
        let ws = wait::waitpid(self.pid, flags).map_err(ProcessErrorKind::NixError)?;
        self.retrieve_exit_status(ws)
    }

    /// Retrieve exit status.
    fn retrieve_exit_status(&mut self, ws: WaitStatus) -> Result<ExitStatus> {
        if let WaitStatus::Signaled(_, Signal::SIGKILL, _) = ws {
            let reason = "container received signal SIGKILL";
            self.status = Some(ExitStatus::new_failure(reason));
        }

        if self.status.is_none() {
            self.retrieve_exit_status_internal_process()?;
        }

        self.logging();
        drop(self.tmpdir.take());

        let s = self.status.clone();
        s.ok_or(Error::ProcessError(ProcessErrorKind::ChildExitStatusGone))
    }

    /// Retrieve the exit status of the internal process from a pipe whose
    /// write end has been closed.
    fn retrieve_exit_status_internal_process(&mut self) -> Result<()> {
        if let Some(mut reader) = self.status_reader.take() {
            if !self.status_reader_noleading {
                let mut request = [0];
                reader
                    .read_exact(&mut request)
                    .map_err(ProcessErrorKind::StdIoError)?;
            }

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
        Ok(())
    }

    /// Logging.
    fn logging(&self) {
        if !log::log_enabled!(log::Level::Debug) {
            return;
        }

        if let Some(status) = &self.status {
            log::debug!("================================");
            log::debug!("Exited: {}", status.reason);

            if let Some(r) = &status.rusage {
                let (rt, ut, st) = (r.real_time, r.user_time, r.system_time);
                log::debug!("Metric:      RealTime: {:>12} sec", rt.as_secs_f64());
                log::debug!("Metric:      UserTime: {:>12} sec", ut.as_secs_f64());
                log::debug!("Metric:       SysTime: {:>12} sec", st.as_secs_f64());
            }

            if let Some(r) = &status.proc_pid_smaps_rollup {
                log::debug!("Metric:           Rss: {:>12} kB", r.rss);
                log::debug!("Metric:  Shared_Dirty: {:>12} kB", r.shared_dirty);
                log::debug!("Metric:  Shared_Clean: {:>12} kB", r.shared_clean);
                log::debug!("Metric: Private_Dirty: {:>12} kB", r.private_dirty);
                log::debug!("Metric: Private_Clean: {:>12} kB", r.private_clean);
                log::debug!("Metric:           Pss: {:>12} kB", r.pss);
                log::debug!("Metric:     Pss_Dirty: {:>12} kB", r.pss_dirty);
                log::debug!("Metric:      Pss_Anon: {:>12} kB", r.pss_anon);
                log::debug!("Metric:      Pss_File: {:>12} kB", r.pss_file);
                log::debug!("Metric:     Pss_Shmem: {:>12} kB", r.pss_shmem);
            }

            if let Some(r) = &status.proc_pid_status {
                log::debug!("Metric:        VmPeak: {:>12} kB", r.vmpeak);
                log::debug!("Metric:        VmSize: {:>12} kB", r.vmsize);
                log::debug!("Metric:         VmHWM: {:>12} kB", r.vmhwm);
                log::debug!("Metric:         VmRSS: {:>12} kB", r.vmrss);
                log::debug!("Metric:        VmData: {:>12} kB", r.vmdata);
                log::debug!("Metric:         VmStk: {:>12} kB", r.vmstk);
                log::debug!("Metric:         VmExe: {:>12} kB", r.vmexe);
                log::debug!("Metric:         VmLib: {:>12} kB", r.vmlib);
                log::debug!("Metric:         VmPTE: {:>12} kB", r.vmpte);
                log::debug!("Metric:        VmSwap: {:>12} kB", r.vmswap);
                log::debug!("Metric:       RssAnon: {:>12} kB", r.rssanon);
                log::debug!("Metric:       RssFile: {:>12} kB", r.rssfile);
                log::debug!("Metric:      RssShmem: {:>12} kB", r.rssshmem);
            }
        } else {
            log::debug!("================================");
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
                Ok(Err(e)) => return Err(ProcessErrorKind::StdIoError(e)),
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
