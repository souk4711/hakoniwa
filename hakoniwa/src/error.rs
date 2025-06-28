/// Error handling with the Result type.
pub type Result<T> = std::result::Result<T, Error>;

/// Error types.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    ProcessError(#[from] ProcessErrorKind),

    #[doc(hidden)]
    #[error("{0}")]
    Unexpected(String),
}

#[derive(thiserror::Error, Debug)]
pub enum ProcessErrorKind {
    #[error(transparent)]
    BincodeDecodeError(#[from] bincode::error::DecodeError),
    #[error(transparent)]
    NixError(#[from] nix::Error),
    #[error(transparent)]
    StdIoError(#[from] std::io::Error),
    #[error("std thread panic")]
    StdThreadPanic,
    #[error("configure the new network namespace failed: {0}")]
    SetupNetworkFailed(String),
    #[error("configure the UID/GID mapping of a user namespace failed: {0}")]
    SetupUGidmapFailed(String),
    #[error("child exit status gone")]
    ChildExitStatusGone,
}
