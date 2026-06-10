/// Manipulates various aspects of the behavior of the container.
#[derive(Hash, Eq, PartialEq, Clone, Copy, Debug)]
pub enum Runctl {
    /// Allow the internal process to gain more privileges than its parent
    /// process. Aka do not set the no_new_privs bit.
    AllowNewPrivs,

    /// Get memory usage through proc_pid_smaps_rollup at exit.
    GetProcPidSmapsRollup,

    /// Get memory usage and status information through proc_pid_status at exit.
    GetProcPidStatus,

    /// Proceed without the specified cgroup resource configuration if initialization
    /// failed, for instance if the systemd socket was not available or systemd
    /// rejected configuration due to permissions.
    IgnoreCgroupSetupFailed,

    /// Fallback when the specific configuration is not applicable. E.g try to
    /// remount a bind mount again after the first attempt failed on source
    /// filesystems that have nodev, noexec, nosuid, etc.
    MountFallback,

    /// Start a new session and acquire the controlling terminal before
    /// executing the program.
    ///
    /// Just prior to `execve`, this calls `setsid(2)` so the internal process
    /// becomes the leader of a new session and process group, then issues
    /// `ioctl(stdin, TIOCSCTTY)` to make the terminal referenced by stdin the
    /// controlling terminal of that session. This is typically what you want
    /// when running an interactive shell so that job control works correctly.
    NewSession,

    /// Mount root dir with read-write access.
    RootdirRW,
}
