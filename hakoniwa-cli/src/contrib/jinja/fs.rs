use minijinja::{Error, ErrorKind::InvalidOperation};
use std::fs;

pub(crate) fn glob(value: String) -> Result<Vec<String>, Error> {
    glob::glob(&value)
        .map_err(|e| Error::new(InvalidOperation, "cannot glob").with_source(e))
        .map(|paths| {
            paths
                .filter_map(|path| path.ok())
                .map(|pathbuf| pathbuf.to_string_lossy().to_string())
                .collect()
        })
}

pub(crate) fn read_link(value: String) -> Result<String, Error> {
    fs::read_link(&value)
        .map_err(|e| Error::new(InvalidOperation, "cannot read link").with_source(e))
        .map(|pathbuf| pathbuf.to_string_lossy().to_string())
}
