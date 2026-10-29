//! Requires a cached setup error to preserve the normal wait/reaping contract.
//!
//! Assumes native Linux user namespaces and exclusive reaping of this child.
//! Overlapping group mappings must be refused by the host setup helper.

use super::configure_command;
use hakoniwa::Container;
use nix::errno::Errno;
use nix::sys::wait::{WaitPidFlag, WaitStatus, waitpid};
use nix::unistd::Pid;
use std::io;

/// Probes reaping after wait and cleans up the original bug before asserting.
#[test]
fn wait_reaps_after_cached_setup_failure() -> io::Result<()> {
    let gid = nix::unistd::getgid().as_raw();
    let mut container = Container::new();
    container.rootfs("/").map_err(io::Error::other)?;
    container.gidmaps(&[(0, gid, 1), (0, gid, 1)]);
    let mut command = container.command("/bin/true");
    configure_command(&mut command)?;
    let mut child = command.spawn().map_err(io::Error::other)?;

    let pid = Pid::from_raw(i32::try_from(child.id()).map_err(io::Error::other)?);
    // Cleanup owners pin the unreaped supervisor after spawn returns.
    // SAFETY: the library has not waited for this exclusively owned child.
    let descriptor = unsafe { libc::syscall(libc::SYS_pidfd_open, pid.as_raw(), 0) };
    if descriptor < 0 {
        let error = io::Error::last_os_error();
        let _ = child.wait();
        return Err(error);
    }
    let descriptor = i32::try_from(descriptor).map_err(io::Error::other)?;
    // SAFETY: pidfd_open returned a fresh descriptor owned by this test.
    let _supervisor =
        unsafe { <std::os::fd::OwnedFd as std::os::fd::FromRawFd>::from_raw_fd(descriptor) };
    let first = child.wait().map_err(io::Error::other)?;
    let second = child.wait().map_err(io::Error::other)?;
    assert_eq!((first.code, first.exit_code), (125, None));
    assert_eq!(second.reason, first.reason);
    let observation = waitpid(pid, Some(WaitPidFlag::WNOHANG));

    if matches!(observation, Ok(WaitStatus::StillAlive)) {
        // Upstream has requested SIGKILL but may not have observed exit yet.
        // Reap it here so the expected failing test leaves no zombie behind.
        let _ = waitpid(pid, None);
    }

    assert!(
        matches!(observation, Err(Errno::ECHILD)),
        "wait returned cached setup failure without reaping: {observation:?}"
    );
    assert!(!child.startup_cleanup_pending());
    Ok(())
}

/// Stops guest execution on refused acquisition, then checks wait's contract.
#[test]
#[ignore = "requires pidfd_open refusal in a disposable process"]
fn refused_acquisition_must_reap_before_wait_returns() -> io::Result<()> {
    use nix::errno::Errno;
    use nix::sys::wait::{WaitPidFlag, WaitStatus, waitpid};
    use nix::unistd::Pid;

    let witness = tempfile::tempdir()?;
    let mount = witness
        .path()
        .to_str()
        .ok_or_else(|| io::Error::other("path encoding"))?;
    let mut container = hakoniwa::Container::new();
    container.rootfs("/").map_err(io::Error::other)?;
    container.bindmount_rw(mount, "/work");
    let mut command = container.command("/bin/sh");
    command.args(["-c", "printf executed > /work/ran"]);
    configure_command(&mut command)?;
    let mut child = command.spawn().map_err(io::Error::other)?;
    let pid = Pid::from_raw(i32::try_from(child.id()).map_err(io::Error::other)?);
    let status = child.wait().map_err(io::Error::other)?;
    assert_eq!(status.code, 125);
    assert!(child.inner_pidfd().is_none());
    let observation = waitpid(pid, Some(WaitPidFlag::WNOHANG));
    if matches!(observation, Ok(WaitStatus::StillAlive)) {
        // Reap our exclusively owned supervisor before reporting the failure.
        waitpid(pid, None).map_err(io::Error::other)?;
    }
    assert!(
        !witness.path().join("ran").exists(),
        "guest ran without acquisition"
    );
    assert!(
        matches!(observation, Err(Errno::ECHILD)),
        "cached pidfd acquisition failure returned from wait without reaping: {observation:?}"
    );
    assert!(!child.startup_cleanup_pending());
    let expected = std::env::var("HAKONIWA_PIDFD_REFUSAL")
        .map_err(io::Error::other)?
        .parse::<i32>()
        .map_err(io::Error::other)?;
    assert_eq!(
        status.reason,
        io::Error::from_raw_os_error(expected).to_string()
    );
    assert_eq!(
        child.wait().map_err(io::Error::other)?.reason,
        status.reason
    );
    Ok(())
}
