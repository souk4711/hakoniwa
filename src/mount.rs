use nix::mount::MsFlags;
use serde::Deserialize;
use std::path::{Path, PathBuf};

#[derive(Deserialize, Clone, Default)]
pub enum MountKind {
    #[default]
    Bind,
    RoBind,
}

#[derive(Deserialize, Clone, Default)]
pub struct Mount {
    pub(crate) host_path: PathBuf,
    pub(crate) container_path: PathBuf,
    pub(crate) kind: MountKind,
}

impl Mount {
    pub(crate) const PROC_DIR: (&'static str, &'static str) = ("proc", "/proc");
    pub(crate) const WORK_DIR: (&'static str, &'static str) = ("hako", "/hako");
    pub(crate) const PUT_OLD_DIR: (&'static str, &'static str) = (".old", "/.old");
    pub(crate) const PUT_OLD_PROC_DIR: (&'static str, &'static str) = (".old_proc", "/.old_proc");

    pub fn new<P1: AsRef<Path>, P2: AsRef<Path>>(
        host_path: P1,
        container_path: P2,
        kind: MountKind,
    ) -> Self {
        Mount {
            host_path: host_path.as_ref().to_path_buf(),
            container_path: container_path.as_ref().to_path_buf(),
            kind,
        }
    }

    pub(crate) fn ms_flags(&self) -> MsFlags {
        match self.kind {
            MountKind::Bind => MsFlags::empty(),
            MountKind::RoBind => MsFlags::MS_RDONLY,
        }
    }
}
