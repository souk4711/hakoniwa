use nix::mount::MsFlags;
use std::fmt;

bitflags::bitflags! {
    /// Mount flags.
    ///
    /// [mount]: https://man7.org/linux/man-pages/man2/mount.2.html
    /// [mount.h]: https://github.com/torvalds/linux/blob/v6.13/include/uapi/linux/mount.h
    #[derive(Hash, Eq, PartialEq, Clone, Copy, Debug)]
    pub struct MountOptions: u32 {
        const RDONLY = 1;
        const NOSUID = 2;
        const NODEV  = 4;
        const NOEXEC = 8;
        const BIND   = 4096;
        const REC    = 16384;
    }
}

impl MountOptions {
    pub(crate) fn to_ms_flags(self) -> MsFlags {
        let mut flags = MsFlags::empty();
        for option in self.iter() {
            match option {
                Self::RDONLY => flags.insert(MsFlags::MS_RDONLY),
                Self::NOSUID => flags.insert(MsFlags::MS_NOSUID),
                Self::NODEV => flags.insert(MsFlags::MS_NODEV),
                Self::NOEXEC => flags.insert(MsFlags::MS_NOEXEC),
                Self::BIND => flags.insert(MsFlags::MS_BIND),
                Self::REC => flags.insert(MsFlags::MS_REC),
                _ => {}
            }
        }
        flags
    }
}

#[derive(Clone, Debug)]
pub(crate) struct Mount {
    pub(crate) source: String,
    pub(crate) target: String,
    pub(crate) fstype: String,
    pub(crate) options: MountOptions,
}

impl fmt::Display for Mount {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{:?} -> {:?}, FsType({}), {:?}",
            self.source,
            self.target,
            self.fstype,
            self.options.to_ms_flags()
        )
    }
}
