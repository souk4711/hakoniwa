use minijinja::{Error, ErrorKind::InvalidOperation};
use std::env;
use std::fs;

// Print all or part of environment
pub(crate) fn printenv(name: String) -> Result<String, Error> {
    match env::var(name) {
        Ok(v) => Ok(v),
        Err(_) => Ok("".to_string()),
    }
}

// Make directories
pub(crate) fn mkdir_p(path: String) -> Result<(), Error> {
    fs::create_dir_all(&path).map_err(|e| {
        let errmsg = format!("mkdir_p({path:?}) => {e}");
        Error::new(InvalidOperation, errmsg)
    })
}

// Create file, or change file timestamps
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

// Search for files in a directory hierarchy
pub(crate) fn find(pattern: String) -> Result<Vec<String>, Error> {
    let paths = glob::glob(&pattern).map_err(|e| {
        let errmsg = format!("find({pattern:?}) => {e}");
        Error::new(InvalidOperation, errmsg)
    })?;
    let paths = paths
        .filter_map(|path| path.ok())
        .map(|pathbuf| pathbuf.to_string_lossy().to_string())
        .collect();
    Ok(paths)
}

// Print resolved symbolic links or canonical file names
pub(crate) fn readlink(path: String) -> Result<String, Error> {
    fs::read_link(&path)
        .map(|pathbuf| pathbuf.to_string_lossy().to_string())
        .map_err(|e| {
            let errmsg = format!("readlink({path:?}) => {e}");
            Error::new(InvalidOperation, errmsg)
        })
}

// Find an XDG user dir
pub(crate) fn xdg_user_dir(name: String) -> Result<String, Error> {
    let path = crate::xdg::user_dir(&name).map_err(|e| {
        let errmsg = format!("xdg_user_dir({name:?}) => {e}");
        Error::new(InvalidOperation, errmsg)
    })?;
    Ok(path)
}
