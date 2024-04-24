pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    ParseConfigurationError(#[from] ParseConfigurationErrorKind),
    #[error("{0}: {1}")]
    PathError(std::path::PathBuf, String),
    #[error("seccomp: {0}")]
    SeccompError(#[from] libseccomp::error::SeccompError),
    #[error("{0}")]
    _ExecutorRunError(String),
}

#[derive(thiserror::Error, Debug)]
pub enum ParseConfigurationErrorKind {
    #[error(transparent)]
    HandlebarsRenderError(#[from] handlebars::RenderError),
    #[error(transparent)]
    TomlError(#[from] toml::de::Error),
}
