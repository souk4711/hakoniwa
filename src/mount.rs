use serde::Deserialize;
use std::path::PathBuf;

#[derive(Clone, Default, Deserialize)]
pub struct Mount {
    pub(crate) source: PathBuf,
    pub(crate) target: PathBuf, // path relative to Executor#rootfs
}

impl Mount {
    pub(crate) const PROC_DIR: (&'static str, &'static str) = ("/proc", "proc");
    pub(crate) const WORK_DIR: (&'static str, &'static str) = ("/hakoniwa", "hakoniwa");
    pub(crate) const PUT_OLD_DIR: (&'static str, &'static str) = ("/.put_old", ".put_old");
    pub(crate) const PUT_OLD_PROC_DIR: (&'static str, &'static str) =
        ("/.put_old_proc", ".put_old_proc");
}
