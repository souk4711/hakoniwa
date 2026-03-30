use std::io::PipeReader;
use std::os::fd::{AsRawFd, RawFd, OwnedFd};

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
            _ => unreachable!("stdio::EndReader::into_pipe_reader"),
        }
    }
}

impl AsRawFd for EndReader {
    fn as_raw_fd(&self) -> RawFd {
        match self {
            EndReader::Pipe(p) => p.as_raw_fd(),
            EndReader::Fd(fd) => fd.as_raw_fd(),
        }
    }
}
