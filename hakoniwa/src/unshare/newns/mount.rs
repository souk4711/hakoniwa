use nix::mount::MsFlags;

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

impl std::fmt::Display for Mount {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self.fstype.as_ref() {
            "devfs" => return write!(f, "  devfs: {}", self.target),
            "tmpfs" => return write!(f, "  tmpfs: {}", self.target),
            "proc" => return write!(f, "   proc: {}", self.target),
            _ => {}
        };

        if self.options & MountOptions::RDONLY == MountOptions::RDONLY {
            write!(f, "bind-ro: {} -> {}", self.source, self.target)
        } else {
            write!(f, "bind-rw: {} -> {}", self.source, self.target)
        }
    }
}
