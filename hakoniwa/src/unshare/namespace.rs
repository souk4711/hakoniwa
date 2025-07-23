use nix::sched::CloneFlags;

/// Linux namespace types.
///
/// [namespaces]: https://man7.org/linux/man-pages/man7/namespaces.7.html
/// [unshare]: https://man7.org/linux/man-pages/man2/unshare.2.html
#[derive(Hash, Eq, PartialEq, Clone, Copy, Debug)]
pub enum Namespace {
    Cgroup,
    Ipc,
    Mount,
    Network,
    Pid,
    User,
    Uts,
}

impl Namespace {
    pub(crate) fn to_clone_flag(self) -> CloneFlags {
        match self {
            Self::Cgroup => CloneFlags::CLONE_NEWCGROUP,
            Self::Ipc => CloneFlags::CLONE_NEWIPC,
            Self::Mount => CloneFlags::CLONE_NEWNS,
            Self::Network => CloneFlags::CLONE_NEWNET,
            Self::Pid => CloneFlags::CLONE_NEWPID,
            Self::User => CloneFlags::CLONE_NEWUSER,
            Self::Uts => CloneFlags::CLONE_NEWUTS,
        }
    }
}

impl std::fmt::Display for Namespace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Cgroup => write!(f, "CGROUP"),
            Self::Ipc => write!(f, "IPC"),
            Self::Mount => write!(f, "MOUNT"),
            Self::Network => write!(f, "NETWORk"),
            Self::Pid => write!(f, "PID"),
            Self::User => write!(f, "USER"),
            Self::Uts => write!(f, "UTS"),
        }
    }
}
