/// Represents an action to be taken on a filter rule.
#[derive(Hash, Eq, PartialEq, Clone, Copy)]
pub enum Action {
    Allow,
    Errno(i32),
    KillProcess,
    KillThread,
    Log,
    Notify,
    Trace(u16),
    Trap,
}
