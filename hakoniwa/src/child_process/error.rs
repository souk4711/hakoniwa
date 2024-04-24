pub(crate) type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub(crate) enum Error {
    #[error(transparent)]
    SeccompError(#[from] libseccomp::error::SeccompError),
    #[error(transparent)]
    PathAbsError(#[from] path_abs::Error),
    #[error(transparent)]
    BincodeError(#[from] BincodeErrorKind),
    #[error("{0}")]
    SyscallError(String),
}

#[derive(thiserror::Error, Debug)]
pub enum BincodeErrorKind {
    #[error(transparent)]
    EncodeError(#[from] bincode::error::EncodeError),
    #[error(transparent)]
    DecodeError(#[from] bincode::error::DecodeError),
}
