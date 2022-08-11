pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct Error(pub String);

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for Error {}

impl From<libseccomp::error::SeccompError> for Error {
    fn from(e: libseccomp::error::SeccompError) -> Self {
        Self(e.to_string())
    }
}
