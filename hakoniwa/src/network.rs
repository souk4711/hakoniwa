//! Configure network.

use nix::unistd::Pid;
use std::process::Command;

use crate::{error::*, Container};

#[derive(Clone, Copy, Debug)]
pub enum Network {
    Pasta,
}

pub(crate) fn configure(container: &Container, child: Pid) -> Result<()> {
    match container.network {
        Some(network) => match network {
            Network::Pasta => configure_pasta(child),
        },
        None => Ok(()),
    }
}

fn configure_pasta(child: Pid) -> Result<()> {
    Command::new("pasta")
        .args(["--config-net", "--no-map-gw", &format!("{}", child)])
        .status()
        .map_err(ProcessErrorKind::StdIoError)?;
    Ok(())
}
