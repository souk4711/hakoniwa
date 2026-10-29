//! Modification notice: Mad Labs, Inc added this file on 2026-10-29.
//! Licensed under LGPL-3.0-only WITH LGPL-3.0-linking-exception.
//!
//! Checks setup failures and unreadable reports after supervisor reaping.
//!
//! Requires native Linux pidfds and /bin/cat. The test owns each child
//! exclusively and cleans it up even when an assertion fails.

use super::{Child, ChildState, ExitStatus};
use crate::{Error, error::ProcessErrorKind, startup};
use nix::sys::wait::waitpid;
use nix::unistd::Pid;
use std::io::{self, PipeWriter, Write};
use std::os::fd::{AsFd, AsRawFd, OwnedFd};
use std::process::{Command, Stdio};

struct PendingFailure {
    child: Child,
    supervisor: OwnedFd,
    gate: Option<PipeWriter>,
    workload: Option<std::process::Child>,
}

impl PendingFailure {
    /// Holds the supervisor alive so a cached report cannot race its exit.
    fn new() -> io::Result<Self> {
        let (input, gate) = std::io::pipe()?;
        let (status_reader, status_writer) = std::io::pipe()?;
        drop(status_writer);
        let mut supervisor = Command::new("/bin/cat")
            .stdin(Stdio::from(input))
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?;
        let identity = (|| {
            let pid = i32::try_from(supervisor.id()).map_err(io::Error::other)?;
            startup::acquire(pid).map(|descriptor| (Pid::from_raw(pid), descriptor))
        })()
        .inspect_err(|_| {
            let _ = supervisor.kill();
            let _ = supervisor.wait();
        })?;
        let child = Child::new(
            identity.0,
            None,
            None,
            None,
            None,
            status_reader,
            false,
            Some(ExitStatus::new_failure("original startup failure")),
            None,
            #[cfg(feature = "cgroups")]
            None,
            #[cfg(feature = "rustslirp")]
            None,
        );

        // The library Child now owns the supervisor's kernel wait.
        drop(supervisor);
        Ok(Self {
            child,
            supervisor: identity.1,
            gate: Some(gate),
            workload: None,
        })
    }

    /// Supplies real workload authority to model failure after acquisition.
    fn acquire_workload(&mut self) -> io::Result<()> {
        let mut workload = Command::new("/bin/sleep").arg("60").spawn()?;
        let descriptor = (|| {
            let pid = i32::try_from(workload.id()).map_err(io::Error::other)?;
            startup::acquire(pid)
        })()
        .inspect_err(|_| {
            let _ = workload.kill();
            let _ = workload.wait();
        })?;
        self.child.inner_pidfd = Some(descriptor);
        self.workload = Some(workload);
        Ok(())
    }

    /// Confirms that the public wait consumed the supervisor's kernel status.
    fn assert_reaped(&self) {
        assert!(matches!(
            waitpid(self.child.pid, Some(nix::sys::wait::WaitPidFlag::WNOHANG)),
            Err(nix::errno::Errno::ECHILD)
        ));
    }
}

impl Drop for PendingFailure {
    fn drop(&mut self) {
        drop(self.gate.take());
        if !matches!(
            self.child.state,
            ChildState::Reaped(_) | ChildState::ReapedWithoutReport
        ) {
            let _ = startup::stop(self.supervisor.as_fd());
            while let Err(nix::errno::Errno::EINTR) = waitpid(self.child.pid, None) {}
        }
        if let Some(workload) = &mut self.workload {
            let _ = workload.kill();
            let _ = workload.wait();
        }
    }
}

/// Polling must not turn an unobserved setup error into completed cleanup.
#[test]
fn pending_setup_poll_waits_for_supervisor_exit() -> io::Result<()> {
    let mut probe = PendingFailure::new()?;
    assert!(probe.child.startup_cleanup_pending());
    assert!(
        probe.child.try_wait().map_err(io::Error::other)?.is_none(),
        "setup failure reported before supervisor exit"
    );
    assert!(probe.child.startup_cleanup_pending());
    drop(probe.gate.take());
    let status = probe.child.wait().map_err(io::Error::other)?;
    assert_eq!(
        (status.code, status.reason.as_str()),
        (125, "original startup failure")
    );
    probe.assert_reaped();
    assert!(!probe.child.startup_cleanup_pending());
    assert_eq!(
        probe.child.wait().map_err(io::Error::other)?.reason,
        status.reason
    );
    assert!(probe.child.try_wait().map_err(io::Error::other)?.is_some());
    Ok(())
}

/// Reaping a failed supervisor must retain the acquired workload identity.
#[test]
fn failed_response_preserves_workload_authority_after_wait() -> io::Result<()> {
    let mut probe = PendingFailure::new()?;
    probe.acquire_workload()?;
    let descriptor = probe
        .child
        .inner_pidfd()
        .ok_or_else(|| io::Error::other("missing pidfd"))?
        .as_raw_fd();
    assert!(
        probe.child.try_wait().map_err(io::Error::other)?.is_none(),
        "setup failure reported before supervisor exit"
    );
    drop(probe.gate.take());
    let status = probe.child.wait().map_err(io::Error::other)?;
    assert_eq!(status.reason, "original startup failure");
    probe.assert_reaped();
    assert_eq!(
        probe.child.inner_pidfd().map(|fd| fd.as_raw_fd()),
        Some(descriptor)
    );
    probe.child.kill().map_err(io::Error::other)?;
    let workload = probe
        .workload
        .as_mut()
        .ok_or_else(|| io::Error::other("missing workload"))?;
    assert!(!workload.wait()?.success());
    assert!(!probe.child.startup_cleanup_pending());
    Ok(())
}

/// Waits for exit readiness without consuming the kernel status under test.
fn supervisor_exited(descriptor: &OwnedFd) -> io::Result<()> {
    let mut event = libc::pollfd {
        fd: descriptor.as_raw_fd(),
        events: libc::POLLIN,
        revents: 0,
    };

    // SAFETY: event is a live pollfd and the borrowed descriptor stays open.
    let ready = unsafe { libc::poll(&mut event, 1, 5000) };
    match ready {
        -1 => Err(io::Error::last_os_error()),
        1 if event.revents & libc::POLLIN != 0 => Ok(()),
        _ => Err(io::Error::other("supervisor did not exit")),
    }
}

/// Kernel reaping must survive errors before FIN, after FIN, and in a report.
#[test]
fn report_errors_preserve_reaping_truth() -> io::Result<()> {
    for poll in [false, true] {
        for (fin_received, bytes) in [(false, &[][..]), (true, &[][..]), (true, &[0x80][..])] {
            let mut probe = PendingFailure::new()?;
            let (reader, mut writer) = std::io::pipe()?;
            writer.write_all(bytes)?;
            drop(writer);
            probe.child.status_reader = Some(reader);
            probe.child.status_reader_noleading = fin_received;
            probe.child.state = ChildState::Running;
            drop(probe.gate.take());
            supervisor_exited(&probe.supervisor)?;
            let result = if poll {
                probe.child.try_wait().map(|_| ())
            } else {
                probe.child.wait().map(|_| ())
            };

            match (fin_received, result) {
                (false, Err(Error::ProcessError(ProcessErrorKind::StdIoError(error)))) => {
                    assert_eq!(error.kind(), io::ErrorKind::UnexpectedEof);
                }
                (true, Err(Error::ProcessError(ProcessErrorKind::PostcardError(_)))) => {}
                (_, other) => {
                    return Err(io::Error::other(format!("unexpected report: {other:?}")));
                }
            }

            probe.assert_reaped();
            assert!(matches!(probe.child.state, ChildState::ReapedWithoutReport));
            assert!(matches!(
                probe.child.wait(),
                Err(Error::ProcessError(ProcessErrorKind::ChildExitStatusGone))
            ));
            assert!(matches!(
                probe.child.try_wait(),
                Err(Error::ProcessError(ProcessErrorKind::ChildExitStatusGone))
            ));
        }
    }

    Ok(())
}

/// Losing a report must not revoke stable authority over a surviving workload.
#[test]
fn report_error_preserves_workload_authority() -> io::Result<()> {
    let mut probe = PendingFailure::new()?;
    probe.acquire_workload()?;
    let descriptor = probe.child.inner_pidfd().map(|fd| fd.as_raw_fd());
    probe.child.state = ChildState::Running;
    probe.child.status_reader_noleading = true;
    drop(probe.gate.take());
    assert!(matches!(
        probe.child.wait(),
        Err(Error::ProcessError(ProcessErrorKind::PostcardError(_)))
    ));
    probe.assert_reaped();
    assert!(matches!(probe.child.state, ChildState::ReapedWithoutReport));
    assert!(descriptor.is_some());
    assert_eq!(
        probe.child.inner_pidfd().map(|fd| fd.as_raw_fd()),
        descriptor
    );
    probe.child.kill().map_err(io::Error::other)?;
    let workload = probe
        .workload
        .as_mut()
        .ok_or_else(|| io::Error::other("missing workload"))?;
    assert!(!workload.wait()?.success());
    Ok(())
}

/// A recycled numeric supervisor PID must never authorize a later signal.
#[test]
fn report_error_prevents_numeric_pid_signal() -> io::Result<()> {
    let mut probe = PendingFailure::new()?;
    probe.acquire_workload()?;
    let descriptor = probe
        .child
        .inner_pidfd
        .take()
        .ok_or_else(|| io::Error::other("missing pidfd"))?;
    let workload = probe
        .workload
        .as_ref()
        .ok_or_else(|| io::Error::other("missing workload"))?;
    let recycled = Pid::from_raw(i32::try_from(workload.id()).map_err(io::Error::other)?);
    probe.child.state = ChildState::Running;
    probe.child.status_reader_noleading = true;
    drop(probe.gate.take());
    assert!(matches!(
        probe.child.wait(),
        Err(Error::ProcessError(ProcessErrorKind::PostcardError(_)))
    ));
    probe.assert_reaped();
    assert!(matches!(probe.child.state, ChildState::ReapedWithoutReport));
    probe.child.pid = recycled;
    probe.child.kill().map_err(io::Error::other)?;
    let mut event = libc::pollfd {
        fd: descriptor.as_raw_fd(),
        events: libc::POLLIN,
        revents: 0,
    };

    // SAFETY: event is initialized and the test owns the live pidfd throughout.
    let ready = unsafe { libc::poll(&mut event, 1, 100) };
    assert_eq!(ready, 0, "kill signaled an unrelated numeric PID");
    Ok(())
}
