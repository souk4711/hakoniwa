bitflags::bitflags! {
    /// NET access flags.
    #[derive(Hash, Eq, PartialEq, Clone, Copy, Debug)]
    pub struct Access: u64 {
        const    TCP_BIND = 1;
        const TCP_CONNECT = 1 << 1;
    }
}
