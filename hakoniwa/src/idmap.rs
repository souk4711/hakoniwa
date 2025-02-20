use std::fmt;

#[derive(Clone)]
pub(crate) struct IdMap {
    pub(crate) container_id: u32,
    pub(crate) host_id: u32,
    pub(crate) size: u32,
}

impl fmt::Display for IdMap {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {} {}", self.container_id, self.host_id, self.size)
    }
}

impl fmt::Debug for IdMap {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "container_id: {}, host_id: {}, count: {}",
            self.container_id, self.host_id, self.size
        )
    }
}
