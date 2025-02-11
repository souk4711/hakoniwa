use crate::runc::{error::*, nix};
use crate::Container;

pub(crate) fn unshare(container: &Container) -> Result<()> {
    nix::unshare(container.namespaces_to_clone_flags())
}
