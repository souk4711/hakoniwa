use nix::{unistd, Result};
use std::path::Path;

pub fn init(work_dir: &Path) -> Result<()> {
    chdir(work_dir)
}

fn chdir(work_dir: &Path) -> Result<()> {
    match work_dir.as_os_str().len() {
        0 => Ok(()),
        _ => unistd::chdir(work_dir),
    }
}
