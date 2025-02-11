use crate::runc::{error::*, nix};
use crate::Container;

pub(crate) fn setrlimit(container: &Container) -> Result<()> {
    for (k, v) in &container.rlimits {
        nix::setrlimit(k.to_resource(), v.0, v.1)?;
    }
    Ok(())
}
