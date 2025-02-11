use nix::sys::resource::Resource;

/// Resource limit types.
#[derive(Hash, Eq, PartialEq, Clone, Copy)]
pub enum Rlimit {
    As,
    Core,
    Cpu,
    Data,
    Fsize,
    Locks,
    Memlock,
    Msgqueue,
    Nice,
    Nofile,
    Nproc,
    Rss,
    Rtprio,
    Rttime,
    Sigpending,
    Stack,
}

impl Rlimit {
    pub(crate) fn to_resource(self) -> Resource {
        match self {
            Self::As => Resource::RLIMIT_AS,
            Self::Core => Resource::RLIMIT_CORE,
            Self::Cpu => Resource::RLIMIT_CPU,
            Self::Data => Resource::RLIMIT_DATA,
            Self::Fsize => Resource::RLIMIT_FSIZE,
            Self::Locks => Resource::RLIMIT_LOCKS,
            Self::Memlock => Resource::RLIMIT_MEMLOCK,
            Self::Msgqueue => Resource::RLIMIT_MSGQUEUE,
            Self::Nice => Resource::RLIMIT_NICE,
            Self::Nofile => Resource::RLIMIT_NOFILE,
            Self::Nproc => Resource::RLIMIT_NPROC,
            Self::Rss => Resource::RLIMIT_RSS,
            Self::Rtprio => Resource::RLIMIT_RTPRIO,
            Self::Rttime => Resource::RLIMIT_RTTIME,
            Self::Sigpending => Resource::RLIMIT_SIGPENDING,
            Self::Stack => Resource::RLIMIT_STACK,
        }
    }
}
