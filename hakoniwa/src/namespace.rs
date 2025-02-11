use nix::sched::CloneFlags;

/// Namespace types available on Linux.
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
            Namespace::Ipc => CloneFlags::CLONE_NEWIPC,
            Namespace::Network => CloneFlags::CLONE_NEWNET,
            Namespace::Mount => CloneFlags::CLONE_NEWNS,
            Namespace::Pid => CloneFlags::CLONE_NEWPID,
            Namespace::User => CloneFlags::CLONE_NEWUSER,
            Namespace::Uts => CloneFlags::CLONE_NEWUTS,
        }
    }
}
