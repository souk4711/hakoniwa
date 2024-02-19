use nix::mount::MsFlags;
use serde::Deserialize;
use std::path::{Path, PathBuf};

#[derive(Deserialize, Clone, Default, Debug)]
#[serde(deny_unknown_fields)]
pub(crate) struct Mount {
    #[serde(rename = "source")]
    pub(crate) host_path: PathBuf,
    #[serde(rename = "target")]
    pub(crate) container_path: PathBuf,
    #[serde(rename = "fstype")]
    pub(crate) fstype: Option<String>,
    #[serde(rename = "rw")]
    pub(crate) rw: Option<bool>,
}

impl Mount {
    pub(crate) const PROC_DIR: (&'static str, &'static str) = ("proc", "/proc");
    pub(crate) const PUT_OLD_DIR: (&'static str, &'static str) = (".old", "/.old");
    pub(crate) const PUT_OLD_PROC_DIR: (&'static str, &'static str) = (".old_proc", "/.old_proc");

    pub(crate) fn new<P1: AsRef<Path>, P2: AsRef<Path>>(
        host_path: P1,
        container_path: P2,
        fstype: Option<String>,
    ) -> Self {
        Self {
            host_path: host_path.as_ref().to_path_buf(),
            container_path: container_path.as_ref().to_path_buf(),
            fstype,
            ..Default::default()
        }
    }

    pub(crate) fn rw(&mut self, flag: Option<bool>) -> &mut Self {
        self.rw = flag;
        self
    }

    pub(crate) fn ms_flags(&self) -> MsFlags {
        let flags = MsFlags::MS_NOSUID;
        match self.rw {
            Some(true) => flags,             // rw
            _ => flags | MsFlags::MS_RDONLY, // ro
        }
    }
}
