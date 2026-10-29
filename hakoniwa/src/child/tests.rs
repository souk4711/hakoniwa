//! Modification notice: Mad Labs, Inc added this file on 2026-10-29.
//! Licensed under LGPL-3.0-only WITH LGPL-3.0-linking-exception.
//!
//! Checks pending setup reports against a supervisor held behind an input gate.
//!
//! Requires native Linux pidfds and /bin/cat. The test owns each child
//! exclusively and cleans it up even when an assertion fails.

use super::{Child, ChildState, ExitStatus};
use crate::startup;
use nix::sys::wait::waitpid;
use nix::unistd::Pid;
use std::io::{self, PipeWriter};
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
        if !matches!(self.child.state, ChildState::Reaped(_)) {
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
