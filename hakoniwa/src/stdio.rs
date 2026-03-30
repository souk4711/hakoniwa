use std::{
    io::{PipeReader, PipeWriter, pipe},
    os::fd::{AsRawFd, OwnedFd},
};

use crate::error::*;

/// Describes what to do with a standard I/O stream.
#[derive(Debug)]
pub enum Stdio {
    Inherit,
    MakePipe,
    Fd(OwnedFd),
}

impl Stdio {
    /// The child inherits from the corresponding parent descriptor.
    pub fn inherit() -> Self {
        Self::Inherit
    }

    /// A new pipe should be arranged to connect the parent and child processes.
    pub fn piped() -> Self {
        Self::MakePipe
    }

    /// The given file descriptor will be used for the corresponding I/O stream.
    pub fn from_fd(fd: OwnedFd) -> Self {
        Self::Fd(fd)
    }

    /// Converts the given instance into the ends of the pipe it represents. `for_output` should
    /// be set to `true` for instances which represent stdout & stderr pipes.
    ///
    ///  - [Inherit](Stdio::Inherit) returns two None values.
    ///  - [MakePipe](Stdio::MakePipe) returns [EndReader::Pipe] & [EndWriter::Pipe].
    ///  - [Fd](Stdio::Fd) returns [EndReader::Fd] & None if for_output is true, and
    ///    None & [EndWriter::Fd] otherwise.
    ///
    pub(crate) fn into_ends(
        io: Self,
        for_output: bool,
    ) -> Result<(Option<EndReader>, Option<EndWriter>)> {
        Ok(match (io, for_output) {
            (Self::Inherit, _) => (None, None),
            (Self::MakePipe, _) => {
                let pipe = pipe().map_err(ProcessErrorKind::StdIoError)?;
                (Some(EndReader::Pipe(pipe.0)), Some(EndWriter::Pipe(pipe.1)))
            }
            (Self::Fd(fd), false) => (Some(EndReader::Fd(fd)), None),
            (Self::Fd(fd), true) => (None, Some(EndWriter::Fd(fd))),
        })
    }
}

impl From<OwnedFd> for Stdio {
    fn from(fd: OwnedFd) -> Self {
        Self::Fd(fd)
    }
}

/// The readable end of a standard I/O stream.
#[derive(Debug)]
pub(crate) enum EndReader {
    Pipe(PipeReader),
    Fd(OwnedFd),
}

impl EndReader {
    /// Returns the inner [PipeReader] if self is the variant [EndReader::Pipe].
    pub fn into_pipe_reader(self) -> Option<PipeReader> {
        match self {
            EndReader::Pipe(p) => Some(p),
            _ => None,
        }
    }
}

impl AsRawFd for EndReader {
    fn as_raw_fd(&self) -> std::os::unix::prelude::RawFd {
        match self {
            EndReader::Pipe(p) => p.as_raw_fd(),
            EndReader::Fd(fd) => fd.as_raw_fd(),
        }
    }
}

/// The writeable end of a standard I/O stream.
#[derive(Debug)]
pub(crate) enum EndWriter {
    Pipe(PipeWriter),
    Fd(OwnedFd),
}

impl EndWriter {
    /// Returns the inner [PipeWriter] if self is the variant [EndWriter::Pipe].
    pub fn into_pipe_writer(self) -> Option<PipeWriter> {
        match self {
            EndWriter::Pipe(p) => Some(p),
            _ => None,
        }
    }
}

impl AsRawFd for EndWriter {
    fn as_raw_fd(&self) -> std::os::unix::prelude::RawFd {
        match self {
            EndWriter::Pipe(p) => p.as_raw_fd(),
            EndWriter::Fd(fd) => fd.as_raw_fd(),
        }
    }
}
