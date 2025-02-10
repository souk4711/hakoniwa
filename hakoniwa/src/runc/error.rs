pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("{0}")]
    NixError(String),
    #[error(transparent)]
    NulError(#[from] std::ffi::NulError),
}
