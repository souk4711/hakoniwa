pub(crate) type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub(crate) enum Error {
    #[error("{0}")]
    HakoniwaError(#[from] hakoniwa::Error),
    #[error("{0}: {1}")]
    FileIoError(std::path::PathBuf, String),
}
