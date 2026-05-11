mod network;
mod pasta;

use crate::{Container, error::*};
use nix::unistd::Pid;
use std::os::fd::RawFd;

pub use network::Network;
pub use pasta::Pasta;

#[cfg(feature = "rustslirp")]
pub(crate) mod rustslirp;

pub(crate) enum SetupStatus {
    None,
    #[cfg(feature = "rustslirp")]
    RustSlirpTapFd(RawFd),
}

pub(crate) fn mainp_setup(container: &Container, child: Pid) -> Result<SetupStatus> {
    let network = &container
        .network
        .clone()
        .expect("Container#network is some");
    let status = match network {
        Network::Pasta(pasta) => Pasta::mainp_setup(pasta, child)?,
        #[cfg(feature = "rustslirp")]
        Network::RustSlirp(rustslirp) => rustslirp::RustSlirp::mainp_setup(rustslirp, child)?,
    };

    log::debug!("================================");
    Ok(status)
}
