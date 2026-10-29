//! Pins the inner process before the supervisor releases its startup gate.
//!
//! Assumes the supervisor exclusively owns the unreaped inner PID in the host
//! PID namespace. The host must not install a foreign reaper or SIGCHLD ignore.
//! A closed gate prevents user code from running even when parent-death setup
//! has not happened. Numeric PIDs are used only during that ownership interval.

use std::io::{self, Read, Write};
use std::os::fd::{AsFd, AsRawFd, BorrowedFd, FromRawFd, OwnedFd};

/// A completed startup carries stable cancellation authority over the inner PID.
#[derive(Debug)]
pub(crate) enum Setup {
    Exited,
    Ready(OwnedFd),
    Failed {
        descriptor: OwnedFd,
        error: io::Error,
    },
}

/// Acquire stable identity while the supervisor has not reaped or released it.
pub(crate) fn acquire(pid: i32) -> io::Result<OwnedFd> {
    // SAFETY: pidfd_open takes an owned unreaped PID and flags, with no pointers.
    let descriptor = unsafe { libc::syscall(libc::SYS_pidfd_open, pid, 0) };

    if descriptor < 0 {
        return Err(io::Error::last_os_error());
    }

    let descriptor = i32::try_from(descriptor).map_err(io::Error::other)?;
    // SAFETY: the successful syscall returned a fresh descriptor owned here.
    Ok(unsafe { OwnedFd::from_raw_fd(descriptor) })
}

/// Pin the gated inner process and acknowledge only after acquisition succeeds.
pub(crate) fn accept(reader: &mut impl Read, writer: &mut impl Write) -> io::Result<Setup> {
    let mut encoded = [0; 4];
    reader.read_exact(&mut encoded)?;
    let pid = i32::from_le_bytes(encoded);

    if pid <= 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "invalid inner PID",
        ));
    }

    let descriptor = acquire(pid)?;
    let response = (|| -> io::Result<u8> {
        writer.write_all(&[1])?;
        let mut completed = [0];
        reader.read_exact(&mut completed)?;
        Ok(completed[0])
    })();

    match response {
        Ok(crate::runc::SETUP_SUCCESS) => Ok(Setup::Ready(descriptor)),
        response => {
            let error = response.err().unwrap_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidData, "inner startup was rejected")
            });

            // A user logger can panic. Keep the acquired descriptor available
            // to Child even when stopping fails; the native owner observes it.
            let _ = stop(descriptor.as_fd());

            // A failed final response may follow release. Carry authority into
            // Child even when SIGKILL fails; a stop request cannot prove exit.
            Ok(Setup::Failed { descriptor, error })
        }
    }
}

/// Deliver SIGKILL through a pinned identity, including after the PID exits.
pub(crate) fn stop(descriptor: BorrowedFd<'_>) -> io::Result<()> {
    // SAFETY: the borrowed pidfd stays open. Null siginfo requests ordinary
    // signal delivery and no userspace memory is accessed.
    let result = unsafe {
        libc::syscall(
            libc::SYS_pidfd_send_signal,
            descriptor.as_raw_fd(),
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
