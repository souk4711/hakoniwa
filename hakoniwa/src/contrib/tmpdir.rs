use fastrand::alphanumeric;
use std::{
    env, fs, iter,
    path::{Path, PathBuf},
};

pub struct Dir {
    path: PathBuf,
}

impl Dir {
    fn new<P: AsRef<Path>>(path: P) -> Result<Self, std::io::Error> {
        let path = path.as_ref().to_path_buf();
        fs::create_dir_all(&path)?;
        Ok(Self { path })
    }
}

impl Drop for Dir {
    fn drop(&mut self) {
        _ = fs::remove_dir_all(&self.path);
    }
}

pub fn new<P: AsRef<Path>>(path: P) -> Result<Dir, std::io::Error> {
    Dir::new(path)
}

pub fn random_name(prefix: &str) -> PathBuf {
    let name: String = iter::repeat_with(alphanumeric).take(8).collect();
    let name = format!("{}-{}", prefix, name);
    env::temp_dir().join(name)
}
