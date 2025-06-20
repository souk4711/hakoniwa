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
    #[error("setup [ug]idmap failed")]
    SetupUGidmapFailed,
    #[error(transparent)]
    StdIoError(#[from] std::io::Error),
    #[cfg(feature = "landlock")]
    #[error("landlock \"{0}\" feature requires minimum kernel version {1}: {2}")]
    LandlockFeatureUnsupported(String, String, String),
    #[cfg(feature = "landlock")]
    #[error("landlock path must be exist: {0}")]
    LandlockPathMustBeAbsolute(String),
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
