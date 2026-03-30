use std::io::PipeWriter;
use std::os::fd::{AsRawFd, OwnedFd, RawFd};

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
            _ => unreachable!("stdio::EndWriter::into_pipe_writer"),
        }
    }
}

impl AsRawFd for EndWriter {
    fn as_raw_fd(&self) -> RawFd {
        match self {
            EndWriter::Pipe(p) => p.as_raw_fd(),
            EndWriter::Fd(fd) => fd.as_raw_fd(),
        }
    }
}
