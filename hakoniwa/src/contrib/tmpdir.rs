use fastrand::alphanumeric;
use std::{
    env, fs, iter,
    path::{Path, PathBuf},
};

pub struct Dir {
    path: PathBuf,
    auto_remove: bool,
}

impl Dir {
    fn new<P: AsRef<Path>>(path: P) -> Result<Self, std::io::Error> {
        let path = path.as_ref().to_path_buf();
        let auto_remove = if path.exists() {
            false
        } else {
            fs::create_dir_all(&path)?;
            true
        };
        Ok(Self { path, auto_remove })
    }
}

impl Drop for Dir {
    fn drop(&mut self) {
        if self.auto_remove {
            _ = fs::remove_dir_all(&self.path);
        }
    }
}

pub(crate) fn new<P: AsRef<Path>>(path: P) -> Result<Dir, std::io::Error> {
    Dir::new(path)
}

pub(crate) fn pathname(prefix: &str) -> PathBuf {
    let name: String = iter::repeat_with(alphanumeric).take(8).collect();
    let name = format!("{}-{}", prefix, name);
    env::temp_dir().join(name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auto_remove_true() {
        let path = pathname("hakoniwa");

        {
            let dir = new(path.clone()).unwrap();
            assert_eq!(dir.auto_remove, true);
            assert_eq!(path.exists(), true);
        }
        assert_eq!(path.exists(), false);
    }

    #[test]
    fn test_auto_remove_false() {
        let path = pathname("hakoniwa");
        _ = fs::create_dir_all(path.clone());

        {
            let dir = new(path.clone()).unwrap();
            assert_eq!(dir.auto_remove, false);
            assert_eq!(path.exists(), true)
        }
        assert_eq!(path.exists(), true)
    }
}
