use std::path::Path;

pub(crate) fn exists(value: String) -> bool {
    Path::new(&value).try_exists().is_ok_and(|v| v)
}

pub(crate) fn is_symlink(value: String) -> bool {
    Path::new(&value).is_symlink()
}
