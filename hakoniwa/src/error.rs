/// Error handling with the Result type.
pub type Result<T> = std::result::Result<T, Error>;

/// Error types.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    ProcessError(#[from] ProcessErrorKind),

    #[error("{0}")]
    UnError(String),
}

#[derive(thiserror::Error, Debug)]
pub enum ProcessErrorKind {
    #[error(transparent)]
    PostcardError(#[from] postcard::Error),
    #[error(transparent)]
    NixError(#[from] nix::Error),
    #[error(transparent)]
    StdIoError(#[from] std::io::Error),
    #[error("thread panic")]
    StdThreadPanic,
    #[error("configure the UID/GID mapping of a user namespace failed: {0}")]
    SetupUGidmapFailed(String),
    #[error("configure the new network namespace failed: {0}")]
    SetupNetworkFailed(String),
    #[cfg(feature = "cgroups")]
    #[error(transparent)]
    SetupCgroupsFailed(#[from] crate::cgroups::Error),
    #[error("container setup for closure failed: {0}")]
    SetupClosuresFailed(String),
    #[error("child exit status gone")]
    ChildExitStatusGone,
}
