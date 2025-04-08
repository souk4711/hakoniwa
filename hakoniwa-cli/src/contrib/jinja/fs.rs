use minijinja::{Error, ErrorKind::InvalidOperation};
use std::fs;
use std::path::Path;

pub(crate) fn is_dir(value: String) -> bool {
    Path::new(&value).is_dir()
}

pub(crate) fn is_file(value: String) -> bool {
    Path::new(&value).is_file()
}

pub(crate) fn is_symlink(value: String) -> bool {
    Path::new(&value).is_symlink()
}

pub(crate) fn read_link(value: String) -> Result<String, Error> {
    fs::read_link(&value)
        .map_err(|e| Error::new(InvalidOperation, "cannot read link").with_source(e))
        .map(|v| v.as_path().to_string_lossy().to_string())
}
