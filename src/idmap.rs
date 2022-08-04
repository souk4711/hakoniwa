use std::fmt::{Display, Formatter, Result};

#[derive(Default)]
pub struct IDMap {
    pub(crate) container_id: libc::uid_t,
    pub(crate) host_id: libc::uid_t,
    pub(crate) size: u32,
}

impl Display for IDMap {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{} {} {}", self.container_id, self.host_id, self.size)
    }
}
