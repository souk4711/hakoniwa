/// Manipulates various aspects of the behavior of the container.
#[derive(Hash, Eq, PartialEq, Clone, Copy, Debug)]
pub enum Runctl {
    /// Mount root dir with read-write access.
    RootdirRW,

    /// Fallback when the specific configuration is not applicable. E.g try to
    /// remount a bind mount again after the first attempt failed on source
    /// filesystems that have nodev, noexec, nosuid, etc.
    MountFallback,

    /// Get memory usage through proc_pid_smaps_rollup at exit.
    GetProcPidSmapsRollup,

    /// Get memory usage and status information through proc_pid_status at exit.
    GetProcPidStatus,

    /// Allow the internal process to gain more privileges than its parent
    /// process. Aka do not set the no_new_privs bit.
    AllowNewPrivs,
}
