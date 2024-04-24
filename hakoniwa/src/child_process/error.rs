pub(crate) type Result<T> = std::result::Result<T, Error>;

#[allow(clippy::enum_variant_names)]
#[derive(thiserror::Error, Debug)]
pub(crate) enum Error {
    #[error(transparent)]
    SeccompError(#[from] libseccomp::error::SeccompError),
    #[error(transparent)]
    BincodeError(#[from] BincodeErrorKind),
    #[error("{0}")]
    SyscallError(String),
}

#[derive(thiserror::Error, Debug)]
pub(crate) enum BincodeErrorKind {
    #[error(transparent)]
    EncodeError(#[from] bincode::error::EncodeError),
    #[error(transparent)]
    DecodeError(#[from] bincode::error::DecodeError),
}
