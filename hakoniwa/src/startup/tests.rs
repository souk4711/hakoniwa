//! Modification notice: Mad Labs, Inc added this file on 2026-10-29.
//! Licensed under LGPL-3.0-only WITH LGPL-3.0-linking-exception.
//!
//! Checks that failed startup responses retain a real acquired pidfd.
//!
//! Requires native Linux pidfds and /bin/sleep. The test guard reaps each
//! workload after the protocol's stop request, even when an assertion fails.

use std::io::{self, Cursor, Write};
use std::os::fd::{AsFd, AsRawFd};

use crate::startup;

struct Workload(std::process::Child);

impl Workload {
    fn start() -> io::Result<Self> {
        std::process::Command::new("/bin/sleep")
            .arg("60")
            .spawn()
            .map(Self)
    }

    fn encoded(&self, response: &[u8]) -> io::Result<Cursor<Vec<u8>>> {
        let pid = i32::try_from(self.0.id()).map_err(io::Error::other)?;
        Ok(Cursor::new(
            [pid.to_le_bytes().as_slice(), response].concat(),
        ))
    }
}

impl Drop for Workload {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

struct RefusedWriter;

impl Write for RefusedWriter {
    fn write(&mut self, _: &[u8]) -> io::Result<usize> {
        Err(io::Error::new(
            io::ErrorKind::BrokenPipe,
            "injected closed host control",
        ))
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// Requires the acquired identity to survive an acknowledgment write failure.
#[test]
fn acknowledgment_failure_retains_descriptor() -> io::Result<()> {
    let mut workload = Workload::start()?;
    let mut input = workload.encoded(&[])?;
    let result = startup::accept(&mut input, &mut RefusedWriter)?;
    let startup::Setup::Failed { descriptor, error } = result else {
        return Err(io::Error::other("acquired authority was not returned"));
    };

    assert_eq!(error.kind(), io::ErrorKind::BrokenPipe);
    assert!(descriptor.as_fd().as_raw_fd() >= 0);
    assert!(!workload.0.wait()?.success());
    Ok(())
}

/// Requires the acquired identity to survive EOF after acknowledgment.
#[test]
fn response_eof_retains_descriptor() -> io::Result<()> {
    let mut workload = Workload::start()?;
    let mut input = workload.encoded(&[])?;
    let mut acknowledgment = Vec::new();
    let result = startup::accept(&mut input, &mut acknowledgment)?;
    let startup::Setup::Failed { descriptor, error } = result else {
        return Err(io::Error::other("acquired authority was not returned"));
    };

    assert_eq!(acknowledgment, [1]);
    assert_eq!(error.kind(), io::ErrorKind::UnexpectedEof);
    assert!(descriptor.as_fd().as_raw_fd() >= 0);
    assert!(!workload.0.wait()?.success());
    Ok(())
}

/// Rejects an invalid final byte without losing ownership of the pinned child.
#[test]
fn rejected_final_byte_retains_descriptor() -> io::Result<()> {
    let mut workload = Workload::start()?;
    let mut input = workload.encoded(&[0])?;
    let mut acknowledgment = Vec::new();
    let result = startup::accept(&mut input, &mut acknowledgment)?;
    let startup::Setup::Failed { descriptor, error } = result else {
        return Err(io::Error::other("acquired authority was not returned"));
    };

    assert_eq!(error.kind(), io::ErrorKind::InvalidData);
    assert!(descriptor.as_fd().as_raw_fd() >= 0);
    assert!(!workload.0.wait()?.success());
    Ok(())
}
