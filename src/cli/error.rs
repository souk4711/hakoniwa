pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("{0}")]
    HakoniwaError(#[from] crate::Error),
    #[error("{0}: {1}")]
    FileIoError(std::path::PathBuf, String),
}
