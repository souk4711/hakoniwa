//! Checks that cancellation survives a workload clearing its parent-death signal.
//!
//! Requires native Linux user and mount namespaces and /usr/bin/python3.
//! An external watchdog must bound the test process if kernel teardown stalls.

use hakoniwa::{Child, Container};
use std::io;
use std::time::{Duration, Instant};

struct Guard(Child);

impl Drop for Guard {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

/// Waits for a guest-written witness before challenging cancellation authority.
fn wait_ready(path: &std::path::Path) -> io::Result<()> {
    let deadline = Instant::now() + Duration::from_secs(5);

    while !path.exists() {
        if Instant::now() >= deadline {
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "guest did not clear PDEATHSIG",
            ));
        }

        std::thread::sleep(Duration::from_millis(10));
    }

    Ok(())
}

/// Waits for the guest witness, then requires the pinned inner process to exit.
#[test]
fn clearing_pdeathsig_does_not_remove_host_cancellation() -> io::Result<()> {
    let dir = tempfile::tempdir()?;
    let source = dir
        .path()
        .to_str()
        .ok_or_else(|| io::Error::other("non-UTF8 temp path"))?;
    let mut container = Container::new();
    container.rootfs("/").map_err(io::Error::other)?;
    container.bindmount_rw(source, "/work");
    let program = "import ctypes,pathlib,time; assert ctypes.CDLL(None).prctl(1,0,0,0,0)==0; pathlib.Path('/work/ready').write_text('cleared'); time.sleep(60)";
    let mut child = Guard(
        container
            .command("/usr/bin/python3")
            .args(["-c", program])
            .spawn()
            .map_err(io::Error::other)?,
    );
    wait_ready(&dir.path().join("ready"))?;
    let descriptor = child
        .0
        .inner_pidfd()
        .ok_or_else(|| io::Error::other("no inner pidfd"))?;
    let descriptor = std::os::fd::AsRawFd::as_raw_fd(&descriptor);
    child.0.kill().map_err(io::Error::other)?;
    let mut poll = libc::pollfd {
        fd: descriptor,
        events: libc::POLLIN,
        revents: 0,
    };

    // SAFETY: poll points to one initialized entry; the Child retains its fd.
    let observed = unsafe { libc::poll(&raw mut poll, 1, 5000) };
    assert_eq!(observed, 1, "the pinned inner process must exit");
    assert_ne!(poll.revents & libc::POLLIN, 0);
    child.0.wait().map_err(io::Error::other)?;
    Ok(())
}
