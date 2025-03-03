pub type Result<T> = std::result::Result<T, Error>;

#[allow(clippy::enum_variant_names)]
#[derive(thiserror::Error, Debug)]
pub(crate) enum Error {
    #[error("{0}")]
    NixError(String),
    #[error(transparent)]
    NulError(#[from] std::ffi::NulError),
    #[error("mount source path must be absolute: {0}")]
    MountSourcePathMustBeAbsolute(String),
    #[error("mount target path must be absolute: {0}")]
    MountTargetPathMustBeAbsolute(String),
    #[error("mount procfs requires a new PID namespace")]
    MountProcfsEPERM,
    #[cfg(feature = "seccomp")]
    #[error(transparent)]
    SeccompError(#[from] libseccomp::error::SeccompError),
}
