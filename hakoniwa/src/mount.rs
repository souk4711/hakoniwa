use nix::mount::MsFlags;
use serde::Deserialize;
use std::path::{Path, PathBuf};

#[derive(Deserialize, Clone, Default, Debug)]
pub enum MountType {
    #[serde(rename = "bind")]
    Bind,
    #[default]
    #[serde(rename = "ro-bind")]
    RoBind,
}

#[derive(Deserialize, Clone, Default, Debug)]
pub struct Mount {
    #[serde(default, rename = "type")]
    pub(crate) r#type: MountType,
    #[serde(rename = "source")]
    pub(crate) host_path: PathBuf,
    #[serde(rename = "target")]
    pub(crate) container_path: PathBuf,
}

impl Mount {
    pub(crate) const TMP_DIR: (&'static str, &'static str) = ("tmp", "/tmp");
    pub(crate) const PROC_DIR: (&'static str, &'static str) = ("proc", "/proc");
    pub(crate) const WORK_DIR: (&'static str, &'static str) = ("hako", "/hako");
    pub(crate) const PUT_OLD_DIR: (&'static str, &'static str) = (".old", "/.old");
    pub(crate) const PUT_OLD_PROC_DIR: (&'static str, &'static str) = (".old_proc", "/.old_proc");

    pub fn new<P1: AsRef<Path>, P2: AsRef<Path>>(
        host_path: P1,
        container_path: P2,
        r#type: MountType,
    ) -> Self {
        Self {
            host_path: host_path.as_ref().to_path_buf(),
            container_path: container_path.as_ref().to_path_buf(),
            r#type,
        }
    }

    pub(crate) fn ms_rdonly_flag(&self) -> MsFlags {
        match self.r#type {
            MountType::Bind => MsFlags::empty(),
            MountType::RoBind => MsFlags::MS_RDONLY,
        }
    }
}
