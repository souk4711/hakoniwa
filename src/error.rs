pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
#[error("{0}")]
pub enum Error {
    ParseConfigurationError(String),
    #[error("{0}: {1}")]
    PathError(std::path::PathBuf, String),
}

impl From<handlebars::RenderError> for Error {
    fn from(e: handlebars::RenderError) -> Self {
        Self::ParseConfigurationError(e.to_string())
    }
}

impl From<toml::de::Error> for Error {
    fn from(e: toml::de::Error) -> Self {
        Self::ParseConfigurationError(e.to_string())
    }
}
