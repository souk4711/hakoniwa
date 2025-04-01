bitflags::bitflags! {
    /// NET access flags.
    #[derive(Hash, Eq, PartialEq, Clone, Copy, Debug)]
    pub struct Access: u64 {
        const    TCP_BIND = 1;
        const TCP_CONNECT = 1 << 1;
    }
}

impl std::fmt::Display for Access {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut str = vec![];
        for e in [Self::TCP_BIND, Self::TCP_CONNECT] {
            str.push(match *self & e {
                Self::TCP_BIND => "tcp.bind",
                Self::TCP_CONNECT => "tcp.connect",
                _ => continue,
            });
        }
        write!(f, "{}", str.join(" | "))
    }
}
