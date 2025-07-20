use super::{error::*, sys};
use crate::Container;

pub(crate) fn setrlimit(container: &Container) -> Result<()> {
    for (k, v) in &container.rlimits {
        sys::setrlimit(k.to_resource(), v.0, v.1)?;
    }
    Ok(())
}
