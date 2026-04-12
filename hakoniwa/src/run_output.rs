use std::fmt;
use std::path::PathBuf;

use crate::ExitStatus;

pub struct ContainerContext {
    pub root: PathBuf,
}

impl fmt::Debug for ContainerContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ContainerContext")
            .field("root", &self.root)
            .finish()
    }
}

pub struct RunOutput<T> {
    pub status: ExitStatus,
    pub data: T,
}

impl<T: fmt::Debug> fmt::Debug for RunOutput<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RunOutput")
            .field("status", &self.status)
            .field("data", &self.data)
            .finish()
    }
}
