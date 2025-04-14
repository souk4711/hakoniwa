use minijinja::{Error, ErrorKind::InvalidOperation};
use std::fs;

pub(crate) fn glob(value: String) -> Result<Vec<String>, Error> {
    glob::glob(&value)
        .map(|paths| {
            paths
                .filter_map(|path| path.ok())
                .map(|pathbuf| pathbuf.to_string_lossy().to_string())
                .collect()
        })
        .map_err(|e| {
            let errmsg = format!("cannot glob: {:?}", value);
            Error::new(InvalidOperation, errmsg).with_source(e)
        })
}

pub(crate) fn mkdir(value: String) -> Result<(), Error> {
    fs::create_dir_all(&value).map_err(|e| {
        let errmsg = format!("cannot mkdir: {:?}", value);
        Error::new(InvalidOperation, errmsg).with_source(e)
    })
}

pub(crate) fn read_link(value: String) -> Result<String, Error> {
    fs::read_link(&value)
        .map(|pathbuf| pathbuf.to_string_lossy().to_string())
        .map_err(|e| {
            let errmsg = format!("cannot read link: {:?}", value);
            Error::new(InvalidOperation, errmsg).with_source(e)
        })
}
