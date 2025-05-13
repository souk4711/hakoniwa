use std::path::Path;

pub(crate) fn exists(path: String) -> bool {
    Path::new(&path).try_exists().is_ok_and(|v| v)
}

pub(crate) fn is_symlink(path: String) -> bool {
    Path::new(&path).is_symlink()
}
