pub type Result<T> = std::result::Result<T, Error>;

#[allow(clippy::enum_variant_names)]
#[derive(thiserror::Error, Debug)]
pub(crate) enum Error {
    #[error("{0}")]
    NixError(String),
    #[error(transparent)]
    NulError(#[from] std::ffi::NulError),
}
