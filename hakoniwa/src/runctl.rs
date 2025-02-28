/// Manipulates various aspects of the behavior of the container.
#[derive(Hash, Eq, PartialEq, Clone, Copy)]
pub enum Runctl {
    /// Mount root dir with read-write access.
    RootdirRW,

    /// Fallback when the specific configuration is not applicable. E.g try to
    /// remount a bind mount again after the first attempt failed on source
    /// failed on source filesystems that have nodev, noexec, nosuid, etc.
    MountFallback,
}
