use minijinja::{Error, ErrorKind::InvalidOperation};
use std::{env, fs};

pub(crate) fn findup(name: String) -> Result<String, Error> {
    let cwd = env::current_dir().map_err(|_| {
        let errmsg = format!("findup({name:?}) => current directory does not exist");
        Error::new(InvalidOperation, errmsg)
    })?;
    for ancestor in cwd.ancestors() {
        let path = ancestor.join(&name);
        match path.try_exists() {
            Ok(true) => return Ok(path.to_string_lossy().to_string()),
            _ => continue,
        }
    }
    Ok(String::new())
}

pub(crate) fn glob(pattern: String) -> Result<Vec<String>, Error> {
    let paths = glob::glob(&pattern).map_err(|e| {
        let errmsg = format!("glob({pattern:?}) => {e}");
        Error::new(InvalidOperation, errmsg)
    })?;
    let paths = paths
        .filter_map(|path| path.ok())
        .map(|pathbuf| pathbuf.to_string_lossy().to_string())
        .collect();
    Ok(paths)
}

pub(crate) fn mkdir(path: String) -> Result<(), Error> {
    fs::create_dir_all(&path).map_err(|e| {
        let errmsg = format!("mkdir({path:?}) => {e}");
        Error::new(InvalidOperation, errmsg)
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
            let errmsg = format!("touch({path:?}) => {e}");
            Error::new(InvalidOperation, errmsg)
        })
}

pub(crate) fn read_link(path: String) -> Result<String, Error> {
    fs::read_link(&path)
        .map(|pathbuf| pathbuf.to_string_lossy().to_string())
        .map_err(|e| {
            let errmsg = format!("read_link({path:?}) => {e}");
            Error::new(InvalidOperation, errmsg)
        })
}

pub(crate) fn xdg_user_dir(name: String) -> Result<String, Error> {
    let path = crate::xdg::user_dir(&name).map_err(|e| {
        let errmsg = format!("xdg_user_dir({name:?}) => {e}");
        Error::new(InvalidOperation, errmsg)
    })?;
    Ok(path)
}
