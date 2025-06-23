mod namespace;
mod newnet;
mod newns;
mod newuser;

pub(crate) use newns::{FsMakeDir, FsMakeSymlink, FsOperation, FsWriteFile, Mount};
pub(crate) use newuser::IdMap;

pub use namespace::Namespace;
pub use newnet::{Network, Pasta};
pub use newns::MountOptions;

pub(crate) fn mainp_setup_network(
    container: &crate::Container,
    child: nix::unistd::Pid,
) -> crate::error::Result<()> {
    newnet::mainp_setup(container, child)
}

pub(crate) fn mainp_setup_ugidmap(
    container: &crate::Container,
    child: nix::unistd::Pid,
) -> crate::error::Result<()> {
    newuser::mainp_setup(container, child)
}
