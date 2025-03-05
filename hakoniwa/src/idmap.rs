use std::fmt;

#[derive(Clone, Debug)]
pub(crate) struct IdMap {
    pub(crate) container_id: u32,
    pub(crate) host_id: u32,
    pub(crate) size: u32,
}

impl IdMap {
    pub(crate) fn to_line(&self) -> String {
        format!("{} {} {}\n", self.container_id, self.host_id, self.size)
    }
}

impl fmt::Display for IdMap {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "container_id: {}, host_id: {}, count: {}",
            self.container_id, self.host_id, self.size
        )
    }
}
