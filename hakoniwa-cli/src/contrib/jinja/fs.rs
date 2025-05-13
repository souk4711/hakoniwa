use minijinja::{Error, ErrorKind::InvalidOperation};
use std::{env, fs};

pub(crate) fn findup(name: String) -> Result<String, Error> {
    env::current_dir()
        .map(|cwd| {
            for ancestor in cwd.ancestors() {
                let path = ancestor.join(&name);
                match path.try_exists() {
                    Ok(true) => return path.to_string_lossy().to_string(),
                    _ => continue,
                }
            }
            "".to_string()
        })
        .map_err(|e| {
            let errmsg = format!("findup({:?}) => {}", name, e);
            Error::new(InvalidOperation, errmsg).with_source(e)
        })
}

pub(crate) fn glob(pattern: String) -> Result<Vec<String>, Error> {
    glob::glob(&pattern)
        .map(|paths| {
            paths
                .filter_map(|path| path.ok())
                .map(|pathbuf| pathbuf.to_string_lossy().to_string())
                .collect()
        })
        .map_err(|e| {
            let errmsg = format!("glob({:?}) => {}", pattern, e);
            Error::new(InvalidOperation, errmsg).with_source(e)
        })
}

pub(crate) fn mkdir(path: String) -> Result<(), Error> {
    fs::create_dir_all(&path).map_err(|e| {
        let errmsg = format!("mkdir({:?}) => {}", path, e);
        Error::new(InvalidOperation, errmsg).with_source(e)
    })
}

pub(crate) fn touch(path: String) -> Result<(), Error> {
    fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(false)
        .open(&path)
        .map(|_| ())
        .map_err(|e| {
            let errmsg = format!("touch({:?}) => {}", path, e);
            Error::new(InvalidOperation, errmsg).with_source(e)
        })
}

pub(crate) fn read_link(path: String) -> Result<String, Error> {
    fs::read_link(&path)
        .map(|pathbuf| pathbuf.to_string_lossy().to_string())
        .map_err(|e| {
            let errmsg = format!("read_link({:?}) => {}", path, e);
            Error::new(InvalidOperation, errmsg).with_source(e)
        })
}
