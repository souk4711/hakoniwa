mod end_reader;
mod end_writer;

pub(crate) use end_reader::EndReader;
pub(crate) use end_writer::EndWriter;

use std::fs::File;
use std::io::pipe;
use std::os::fd::OwnedFd;

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

    /// Converts the given instance into the ends of the pipe it represents. `for_output` should
    /// be set to `true` for instances which represent stdout & stderr pipes.
    ///
    ///  - [Inherit](Stdio::Inherit) returns two None values.
    ///  - [MakePipe](Stdio::MakePipe) returns [EndReader::Pipe] & [EndWriter::Pipe].
    ///  - [Fd](Stdio::Fd) returns [EndReader::Fd] & None if for_output is false
    ///  - [Fd](Stdio::Fd) returns None & [EndWriter::Fd] if for_output is true
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

impl From<File> for Stdio {
    /// Converts a [File] into a [Stdio].
    fn from(fd: File) -> Self {
        Self::Fd(fd.into())
    }
}

impl From<OwnedFd> for Stdio {
    /// Takes ownership of a file descriptor and returns a [Stdio] that can attach a stream to it.
    fn from(fd: OwnedFd) -> Self {
        Self::Fd(fd)
    }
}
