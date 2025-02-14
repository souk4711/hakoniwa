use nix::mount::MsFlags;

bitflags::bitflags! {
    /// Mount flags.
    ///
    /// [mount]: https://man7.org/linux/man-pages/man2/mount.2.html
    #[derive(Hash, Eq, PartialEq, Clone, Copy)]
    pub struct MountOptions: u32 {
        /// Mount read-only.
        const RDONLY = 1;

        /// Ignore suid and sgid bits.
        const NOSUID = 2;

        /// Disallow access to device special files
        const NODEV  = 4;

        /// Disallow program execution.
        const NOEXEC = 8;

        /// Bind directory at different place.
        const BIND   = 4096;

        /// Create a recursive bind mount.
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

#[derive(Clone)]
pub(crate) struct Mount {
    pub(crate) source: String,
    pub(crate) target: String,
    pub(crate) options: MountOptions,
}
