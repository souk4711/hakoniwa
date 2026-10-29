//! Challenges process ownership through the existing public API.
//!
//! Requires native Linux namespaces, pidfds, readable procfs and Python 3.
//! The test independently observes the guest and kills it on failed assertions.
//! Run under an outer watchdog; no foreign SIGCHLD reaper may share the process.

use super::configure_command;
use hakoniwa::{Child, Container};
use std::collections::BTreeSet;
use std::fs;
use std::io;
use std::os::fd::{AsFd, AsRawFd, BorrowedFd, FromRawFd, OwnedFd};
use std::os::unix::fs::MetadataExt;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{Duration, Instant};

// Rootfs discovery compares process-wide temporary directory snapshots.
static WORKLOADS: Mutex<()> = Mutex::new(());

/// Opens a stable process observer while the owner still holds an unreaped PID.
fn pidfd(pid: i32) -> io::Result<OwnedFd> {
    // SAFETY: the syscall has no pointer arguments and returns a new owned fd.
    let raw = unsafe { libc::syscall(libc::SYS_pidfd_open, pid, 0) };

    if raw < 0 {
        return Err(io::Error::last_os_error());
    }

    let raw = i32::try_from(raw).map_err(io::Error::other)?;
    // SAFETY: a successful pidfd_open returned a fresh descriptor owned here.
    Ok(unsafe { OwnedFd::from_raw_fd(raw) })
}

/// Stops only the independently pinned guest, including during panic cleanup.
fn stop(fd: BorrowedFd<'_>) -> io::Result<()> {
    // SAFETY: the fd is live and null siginfo selects ordinary signal delivery.
    let result = unsafe {
        libc::syscall(
            libc::SYS_pidfd_send_signal,
            fd.as_raw_fd(),
            libc::SIGKILL,
            std::ptr::null::<libc::siginfo_t>(),
            0,
        )
    };

    match result {
        0 => Ok(()),
        _ => {
            let error = io::Error::last_os_error();

            match error.raw_os_error() {
                Some(libc::ESRCH) => Ok(()),
                _ => Err(error),
            }
        }
    }
}

/// Polls a stable identity; reaping also counts as completed process exit.
fn exited(fd: BorrowedFd<'_>, timeout_ms: i32) -> io::Result<bool> {
    let mut entry = libc::pollfd {
        fd: fd.as_raw_fd(),
        events: libc::POLLIN,
        revents: 0,
    };

    // SAFETY: entry is one initialized pollfd and its descriptor stays owned.
    let result = unsafe { libc::poll(&raw mut entry, 1, timeout_ms) };

    match result {
        -1 => Err(io::Error::last_os_error()),
        0 => Ok(false),
        _ => Ok(entry.revents & (libc::POLLIN | libc::POLLHUP) != 0),
    }
}

struct ObservedChild {
    child: Child,
    inner: OwnedFd,
    inner_pid: i32,
}

impl ObservedChild {
    /// Pins the single workload before any signal or supervisor reaping occurs.
    fn new(mut child: Child) -> io::Result<Self> {
        let acquire = || -> io::Result<(i32, OwnedFd)> {
            let outer = child.id();
            let path = format!("/proc/{outer}/task/{outer}/children");
            let children = fs::read_to_string(path)?;
            let pids = children.split_whitespace().collect::<Vec<_>>();
            let [pid] = pids.as_slice() else {
                return Err(io::Error::other("expected one unreaped workload"));
            };

            let pid = pid.parse::<i32>().map_err(io::Error::other)?;
            Ok((pid, pidfd(pid)?))
        };

        match acquire() {
            Ok((inner_pid, inner)) => Ok(Self {
                child,
                inner,
                inner_pid,
            }),
            Err(error) => {
                let _ = child.kill();
                let _ = child.wait();
                Err(error)
            }
        }
    }
}

impl Drop for ObservedChild {
    fn drop(&mut self) {
        // The observer signals only here or in the explicit signal-control test.
        // It cannot make a broken Child::kill pass its exit assertion.
        let _ = stop(self.inner.as_fd());
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// Waits for the guest to confirm its parent-death setting before cancellation.
fn ready(path: &Path) -> io::Result<()> {
    let deadline = Instant::now() + Duration::from_secs(5);

    while !path.exists() {
        if Instant::now() >= deadline {
            return Err(io::Error::new(io::ErrorKind::TimedOut, "guest not ready"));
        }

        std::thread::sleep(Duration::from_millis(10));
    }

    Ok(())
}

/// Starts a live workload with an explicit parent-death setting and a witness.
fn workload(directory: &Path, clear: bool) -> io::Result<ObservedChild> {
    let mut container = Container::new();
    container.rootfs("/").map_err(io::Error::other)?;
    container.bindmount_rw(
        directory
            .to_str()
            .ok_or_else(|| io::Error::other("path encoding"))?,
        "/work",
    );

    let setting = if clear { "True" } else { "False" };
    let program = format!(
        "import ctypes,pathlib,time\n\
         if {setting}: assert ctypes.CDLL(None).prctl(1,0,0,0,0)==0\n\
         pathlib.Path('/work/ready').write_text('ready')\n\
         time.sleep(60)"
    );

    let mut command = container.command("/usr/bin/python3");
    command.args(["-c", &program]);
    configure_command(&mut command)?;
    let child = command.spawn().map_err(io::Error::other)?;

    ObservedChild::new(child)
}

/// Requires public cancellation to work after untrusted code clears PDEATHSIG.
#[test]
fn kill_survives_cleared_parent_death_signal() -> io::Result<()> {
    let _isolation = WORKLOADS
        .lock()
        .map_err(|error| io::Error::other(error.to_string()))?;

    let directory = tempfile::tempdir()?;
    let mut observed = workload(directory.path(), true)?;
    ready(&directory.path().join("ready"))?;
    assert!(!exited(observed.inner.as_fd(), 0)?);
    observed.child.kill().map_err(io::Error::other)?;
    assert!(
        exited(observed.inner.as_fd(), 1500)?,
        "Child::kill left the workload alive"
    );
    observed.child.wait().map_err(io::Error::other)?;
    Ok(())
}

/// Shows that the same observer sees normal parent-death cancellation succeed.
#[test]
fn parent_death_signal_is_a_live_control() -> io::Result<()> {
    let _isolation = WORKLOADS
        .lock()
        .map_err(|error| io::Error::other(error.to_string()))?;

    let directory = tempfile::tempdir()?;
    let mut observed = workload(directory.path(), false)?;
    ready(&directory.path().join("ready"))?;
    observed.child.kill().map_err(io::Error::other)?;
    assert!(exited(observed.inner.as_fd(), 1500)?);
    observed.child.wait().map_err(io::Error::other)?;
    Ok(())
}

/// Lists host-side temporary roots without confusing other temporary fixtures.
fn temporary_roots() -> io::Result<BTreeSet<PathBuf>> {
    fs::read_dir(std::env::temp_dir())?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<io::Result<BTreeSet<_>>>()
        .map(|paths| {
            paths
                .into_iter()
                .filter(|path| {
                    path.file_name()
                        .and_then(|name| name.to_str())
                        .is_some_and(|name| name.starts_with("hakoniwa-"))
                })
                .collect()
        })
}

/// Retains the actual namespace root after wait and releases it with Child.
#[test]
fn rootfs_remains_owned_until_child_drop() -> io::Result<()> {
    let _isolation = WORKLOADS
        .lock()
        .map_err(|error| io::Error::other(error.to_string()))?;

    let before = temporary_roots()?;
    let directory = tempfile::tempdir()?;
    let mut observed = workload(directory.path(), false)?;
    ready(&directory.path().join("ready"))?;
    let after = temporary_roots()?;
    let created = after.difference(&before).cloned().collect::<Vec<_>>();
    let [rootfs] = created.as_slice() else {
        return Err(io::Error::other(format!(
            "expected one acquired root: {created:?}"
        )));
    };

    let owned = fs::metadata(rootfs)?;
    let namespace = fs::metadata(format!("/proc/{}/root", observed.inner_pid))?;
    assert_eq!(
        (owned.dev(), owned.ino()),
        (namespace.dev(), namespace.ino())
    );
    observed.child.kill().map_err(io::Error::other)?;
    assert!(exited(observed.inner.as_fd(), 1500)?);
    observed.child.wait().map_err(io::Error::other)?;
    assert!(rootfs.exists(), "wait disposed the still-owned rootfs");
    drop(observed);
    assert!(!rootfs.exists(), "dropping Child retained its rootfs");
    Ok(())
}
