pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    ProcessError(#[from] ProcessErrorKind),
}

#[derive(thiserror::Error, Debug)]
pub enum ProcessErrorKind {
    #[error(transparent)]
    BincodeDecodeError(#[from] bincode::error::DecodeError),
    #[error(transparent)]
    NixError(#[from] nix::Error),
    #[error(transparent)]
    StdIoError(#[from] std::io::Error),
}
