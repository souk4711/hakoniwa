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
    #[error("setup network failed")]
    SetupNetworkFailed,
    #[error(transparent)]
    StdIoError(#[from] std::io::Error),
    #[cfg(feature = "landlock")]
    #[error(transparent)]
    LandlockPathfdError(#[from] landlock::PathFdError),
    #[cfg(feature = "landlock")]
    #[error(transparent)]
    LandlockRulesetError(#[from] landlock::RulesetError),
    #[cfg(feature = "seccomp")]
    #[error(transparent)]
    SeccompError(#[from] libseccomp::error::SeccompError),
}
