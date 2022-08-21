use nix::mount::MsFlags;
use serde::Deserialize;
use std::path::{Path, PathBuf};

#[derive(Deserialize, Clone, Default, Debug)]
pub struct Mount {
    #[serde(rename = "source")]
    pub(crate) host_path: PathBuf,
    #[serde(rename = "target")]
    pub(crate) container_path: PathBuf,
    #[serde(rename = "fstype")]
    pub(crate) fstype: Option<String>,
    #[serde(rename = "rw", default)]
    pub(crate) rd_wr: bool,
}

impl Mount {
    pub(crate) const PROC_DIR: (&'static str, &'static str) = ("proc", "/proc");
    pub(crate) const PUT_OLD_DIR: (&'static str, &'static str) = (".old", "/.old");
    pub(crate) const PUT_OLD_PROC_DIR: (&'static str, &'static str) = (".old_proc", "/.old_proc");

    pub fn new<P1: AsRef<Path>, P2: AsRef<Path>>(
        host_path: P1,
        container_path: P2,
        fstype: Option<String>,
        rd_wr: bool,
    ) -> Self {
        Self {
            host_path: host_path.as_ref().to_path_buf(),
            container_path: container_path.as_ref().to_path_buf(),
            fstype,
            rd_wr,
        }
    }

    pub(crate) fn ms_flags(&self) -> MsFlags {
        match self.rd_wr {
            true => MsFlags::MS_NOSUID,
            _ => MsFlags::MS_NOSUID | MsFlags::MS_RDONLY,
        }
    }
}
