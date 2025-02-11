use nix::sched::CloneFlags;

/// Linux namespace types.
#[derive(Hash, Eq, PartialEq, Clone, Copy)]
pub enum Namespace {
    Ipc,
    Network,
    Mount,
    Pid,
    User,
    Uts,
}

impl Namespace {
    pub(crate) fn to_clone_flag(self) -> CloneFlags {
        match self {
            Self::Ipc => CloneFlags::CLONE_NEWIPC,
            Self::Network => CloneFlags::CLONE_NEWNET,
            Self::Mount => CloneFlags::CLONE_NEWNS,
            Self::Pid => CloneFlags::CLONE_NEWPID,
            Self::User => CloneFlags::CLONE_NEWUSER,
            Self::Uts => CloneFlags::CLONE_NEWUTS,
        }
    }
}
