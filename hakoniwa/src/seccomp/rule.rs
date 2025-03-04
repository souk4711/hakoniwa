use crate::seccomp::{Action, ArgCmp};

#[derive(Clone)]
pub struct Rule {
    pub(crate) action: Action,
    pub(crate) sysname: String,
    pub(crate) argcmps: Vec<ArgCmp>,
}

impl std::fmt::Debug for Rule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let action = self.action;
        let sysname = &self.sysname;
        let argcmps = self
            .argcmps
            .iter()
            .map(|cmp| format!("{:?}", cmp))
            .collect::<Vec<_>>()
            .join(", ");

        if argcmps.is_empty() {
            write!(f, "{}(...) -> {:?}", sysname, action)
        } else {
            write!(f, "{}({}, ...) -> {:?}", sysname, argcmps, action)
        }
    }
}
