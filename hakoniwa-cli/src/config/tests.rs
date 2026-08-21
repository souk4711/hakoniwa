use std::path::Path;

pub(crate) fn path_exists(path: String) -> bool {
    Path::new(&path).try_exists().is_ok_and(|v| v)
}

pub(crate) fn path_is_dir(path: String) -> bool {
    Path::new(&path).is_dir()
}

pub(crate) fn path_is_symlink(path: String) -> bool {
    Path::new(&path).is_symlink()
}
