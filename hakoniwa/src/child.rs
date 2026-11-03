//! Owns the supervisor status and pinned inner process cancellation handle.
//!
//! Assumes exclusive reaping. A pidfd stays safe after numeric PID reuse.
//! The acquired rootfs belongs to Child until disposal. Status decoding alone
//! does not prove that the caller's pinned inner cleanup has completed.

use nix::sys::signal::{self, Signal};
use nix::sys::wait::{self, WaitPidFlag, WaitStatus};
use nix::unistd::Pid;
use serde::{Deserialize, Serialize};
use std::io::prelude::*;
use std::io::{PipeReader, PipeWriter};
use std::os::fd::{AsFd, BorrowedFd, OwnedFd};
use std::thread;
use std::{fmt, str};
use tempfile::TempDir;

use crate::{Command, ProcPidSmapsRollup, ProcPidStatus, Rusage, error::*};

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
    ///
    /// Only initialized when using [crate::Runctl::GetProcPidSmapsRollup].
    pub proc_pid_smaps_rollup: Option<ProcPidSmapsRollup>,

    /// Memory usage and status information of the internal process.
    ///
    /// Only initialized when using [crate::Runctl::GetProcPidStatus].
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
                unreachable!("ExitStatus::from_wait_status");
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

/// Separates a setup report from a supervisor result observed by a kernel wait.
#[derive(Debug)]
enum ChildState {
    Running,
    SetupFailed(ExitStatus),
    Reaped(ExitStatus),
    ReapedWithoutReport,
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
    inner_pidfd: Option<OwnedFd>,
    state: ChildState,
    status_reader: Option<PipeReader>,
    status_reader_noleading: bool,
    _tmpdir: Option<TempDir>,

    /// Only initialized when using [Stdio::piped()](crate::Stdio::piped()).
    pub stdin: Option<PipeWriter>,

    /// Only initialized when using [Stdio::piped()](crate::Stdio::piped()).
    pub stdout: Option<PipeReader>,

    /// Only initialized when using [Stdio::piped()](crate::Stdio::piped()).
    pub stderr: Option<PipeReader>,

    /// .
    #[cfg(feature = "cgroups")]
    cgroup: Option<crate::cgroups::Manager>,

    /// Only initialized when using [crate::RustSlirp].
    #[cfg(feature = "rustslirp")]
    pub rustslirp_tapfd: Option<std::os::fd::RawFd>,
}

impl Child {
    /// Constructor.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        pid: Pid,
        inner_pidfd: Option<OwnedFd>,
        stdin: Option<PipeWriter>,
        stdout: Option<PipeReader>,
        stderr: Option<PipeReader>,
        status_reader: PipeReader,
        status_reader_noleading: bool,
        status: Option<ExitStatus>,
        tmpdir: Option<TempDir>,
        #[cfg(feature = "cgroups")] cgroup: Option<crate::cgroups::Manager>,
        #[cfg(feature = "rustslirp")] rustslirp_tapfd: Option<std::os::fd::RawFd>,
    ) -> Self {
        Self {
            pid,
            inner_pidfd,
            stdin,
            stdout,
            stderr,
            status_reader: Some(status_reader),
            status_reader_noleading,
            state: status.map_or(ChildState::Running, ChildState::SetupFailed),
            _tmpdir: tmpdir,
            #[cfg(feature = "cgroups")]
            cgroup,
            #[cfg(feature = "rustslirp")]
            rustslirp_tapfd,
        }
    }

    /// Returns the OS-assigned process identifier associated with this child.
    pub fn id(&self) -> u32 {
        self.pid.as_raw() as u32
    }

    /// Borrow stable cancellation authority for the inner process.
    ///
    /// Startup acquisition supplies this descriptor, including when a later
    /// response fails. It is absent only if acquisition never completed; it
    /// remains valid after exit and reaping.
    pub fn inner_pidfd(&self) -> Option<BorrowedFd<'_>> {
        self.inner_pidfd.as_ref().map(AsFd::as_fd)
    }

    /// Report rejected setup whose supervisor still owns gated-child cleanup.
    ///
    /// A cached setup report does not prove supervisor exit. This hint remains
    /// true for rejected acquisition until a kernel wait reaps the supervisor.
    pub fn startup_cleanup_pending(&self) -> bool {
        self.inner_pidfd.is_none()
            && matches!(self.state, ChildState::SetupFailed(_))
            && !self.status_reader_noleading
    }

    /// Forces the child process to exit.
    pub fn kill(&mut self) -> Result<()> {
        if let Some(descriptor) = self.inner_pidfd() {
            return crate::startup::stop(descriptor)
                .map_err(ProcessErrorKind::StdIoError)
                .map_err(Into::into);
        }

        // Rejected setup keeps cooperative cleanup with its supervisor.
        // After reaping, the numeric supervisor PID may belong to another process.
        if !matches!(self.state, ChildState::Running) {
            return Ok(());
        }

        _ = signal::kill(self.pid, Signal::SIGKILL);
        Ok(())
    }

    /// Attempts to collect the exit status of the child if it has already exited.
    ///
    /// This function will not block the calling thread and will only check to see
    /// if the child process has exited or not. If the child has exited then on Unix
    /// the process ID is reaped. A decoded exit status is cached for later calls.
    ///
    /// If the child has exited, then Ok(Some(status)) is returned. If the exit
    /// status is not available at this time then Ok(None) is returned. If an error
    /// occurs, then that error is returned.
    ///
    /// If reading or decoding the report fails after reaping, this call returns
    /// that error. Later waits return `ProcessErrorKind::ChildExitStatusGone`
    /// without another kernel wait. Any acquired workload pidfd remains usable.
    ///
    /// Note that unlike wait, this function will not attempt to drop stdin.
    pub fn try_wait(&mut self) -> Result<Option<ExitStatus>> {
        match &self.state {
            ChildState::Reaped(status) => return Ok(Some(status.clone())),
            ChildState::ReapedWithoutReport => {
                return Err(ProcessErrorKind::ChildExitStatusGone.into());
            }
            ChildState::Running | ChildState::SetupFailed(_) => {}
        }

        let flags = Some(WaitPidFlag::WNOHANG);
        let ws = self.wait_supervisor(flags)?;
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
    ///
    /// If reading or decoding the report fails after reaping, this call returns
    /// that error. Later waits return `ProcessErrorKind::ChildExitStatusGone`
    /// without another kernel wait. Any acquired workload pidfd remains usable.
    pub fn wait(&mut self) -> Result<ExitStatus> {
        drop(self.stdin.take());

        match &self.state {
            ChildState::Reaped(status) => return Ok(status.clone()),
            ChildState::ReapedWithoutReport => {
                return Err(ProcessErrorKind::ChildExitStatusGone.into());
            }
            ChildState::Running | ChildState::SetupFailed(_) => {}
        }

        let flags = None;
        let ws = self.wait_supervisor(flags)?;
        self.retrieve_exit_status(ws)
    }

    /// Retries interrupted kernel waits without discarding the pending setup
    /// report or its acquired workload descriptor.
    fn wait_supervisor(&self, flags: Option<WaitPidFlag>) -> Result<WaitStatus> {
        loop {
            match wait::waitpid(self.pid, flags) {
                Err(nix::errno::Errno::EINTR) => continue,
                result => {
                    return result
                        .map_err(ProcessErrorKind::NixError)
                        .map_err(Into::into);
                }
            }
        }
    }

    /// Records kernel reaping before reading the report, so a read or decode
    /// error cannot make a recycled supervisor PID look like a running child.
    fn retrieve_exit_status(&mut self, ws: WaitStatus) -> Result<ExitStatus> {
        let previous = std::mem::replace(&mut self.state, ChildState::ReapedWithoutReport);
        let report = match previous {
            ChildState::SetupFailed(status) | ChildState::Reaped(status) => Ok(status),
            ChildState::Running => match ws {
                WaitStatus::Signaled(_, Signal::SIGKILL, _) => {
                    Ok(ExitStatus::new_failure("container received signal SIGKILL"))
                }
                _ => self.retrieve_exit_status_internal_process(),
            },
            ChildState::ReapedWithoutReport => Err(ProcessErrorKind::ChildExitStatusGone.into()),
        };

        let status = report?;
        self.state = ChildState::Reaped(status.clone());
        drop(self.status_reader.take());
        Self::logging(&status);

        #[cfg(feature = "cgroups")]
        drop(self.cgroup.take());
        Ok(status)
    }

    /// Decodes the supervisor's final report after its kernel wait completed.
    /// The leading FIN may already have been consumed during failed admission.
    fn retrieve_exit_status_internal_process(&mut self) -> Result<ExitStatus> {
        let mut reader = self
            .status_reader
            .take()
            .ok_or(Error::ProcessError(ProcessErrorKind::ChildExitStatusGone))?;

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

        postcard::from_bytes(&encoded)
            .map_err(ProcessErrorKind::PostcardError)
            .map_err(Into::into)
    }

    /// Emits the completed report and metrics without inferring process state.
    fn logging(status: &ExitStatus) {
        if !log::log_enabled!(log::Level::Debug) {
            return;
        }

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
    }

    /// Simultaneously waits for the child to exit and collect all remaining
    /// output on the stdout/stderr handles, returning an `Output` instance.
    ///
    /// The stdin handle to the child process, if any, will be closed before
    /// waiting. This helps avoid deadlock: it ensures that the child does not
    /// block waiting for input from the parent, while the parent waits for
    /// the child to exit.
    ///
    /// If the child was configured with standard output or error wired to
    /// a file descriptor or inherited from the process, then no output will be
    /// returned.
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
                Err(_) => return Err(ProcessErrorKind::StdThreadJoinError),
                Ok(Err(e)) => return Err(ProcessErrorKind::StdIoError(e)),
                Ok(Ok(_)) => {}
            }

            let r = threrr.join();
            match r {
                Err(_) => Err(ProcessErrorKind::StdThreadJoinError),
                Ok(r) => r.map_err(ProcessErrorKind::StdIoError),
            }
        })?;
        Ok(())
    }
}

#[cfg(test)]
mod tests;
