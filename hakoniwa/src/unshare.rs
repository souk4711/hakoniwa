mod namespace;
mod newnet;
mod newns;
mod newuser;

pub(crate) use newnet::SetupStatus as SetupNetworkStatus;
pub(crate) use newns::{FsMakeDir, FsMakeSymlink, FsOperation, FsWriteFile, Mount};
pub(crate) use newuser::IdMap;

pub use namespace::Namespace;
pub use newnet::{Network, Pasta};
pub use newns::MountOptions;

#[cfg(feature = "rustslirp")]
pub use newnet::rustslirp::RustSlirp;
pub use newnet::rustslirp::RustSlirpMode;

pub(crate) fn mainp_setup_network(
    container: &crate::Container,
    child: nix::unistd::Pid,
) -> crate::error::Result<SetupNetworkStatus> {
    newnet::mainp_setup(container, child)
}

pub(crate) fn mainp_setup_ugidmap(
    container: &crate::Container,
    child: nix::unistd::Pid,
) -> crate::error::Result<()> {
    newuser::mainp_setup(container, child)
}
