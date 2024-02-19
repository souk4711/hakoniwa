use std::fmt::{Display, Formatter, Result};

#[derive(Default, Debug)]
pub(crate) struct IDMap {
    pub(crate) container_id: u32,
    pub(crate) host_id: u32,
    pub(crate) size: u32,
}

impl Display for IDMap {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{} {} {}", self.container_id, self.host_id, self.size)
    }
}
