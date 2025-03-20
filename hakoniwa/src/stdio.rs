use os_pipe::{PipeReader, PipeWriter};

use crate::error::*;

/// Describes what to do with a standard I/O stream.
#[derive(Clone, Copy, Debug)]
pub enum Stdio {
    Inherit,
    MakePipe,
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

    /// Create a pipe that arranged to connect the parent and child processes.
    pub(crate) fn make_pipe(io: Self) -> Result<(Option<PipeReader>, Option<PipeWriter>)> {
        Ok(match io {
            Self::Inherit => (None, None),
            Self::MakePipe => {
                let pipe = os_pipe::pipe().map_err(ProcessErrorKind::StdIoError)?;
                (Some(pipe.0), Some(pipe.1))
            }
        })
    }
}
