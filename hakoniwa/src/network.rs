//! Configure network.

mod pasta;

pub use pasta::Pasta;

use nix::unistd::Pid;
use std::process::Command;

use crate::{error::*, Container};

#[derive(Clone, Debug)]
pub enum Network {
    Pasta(Pasta),
}

impl From<Pasta> for Network {
    fn from(val: Pasta) -> Self {
        Self::Pasta(val)
    }
}

pub(crate) fn configure(container: &Container, child: Pid) -> Result<()> {
    match &container.network {
        Some(network) => match network {
            Network::Pasta(pasta) => configure_pasta(pasta, child),
        },
        None => unreachable!(),
    }
}

fn configure_pasta(pasta: &Pasta, child: Pid) -> Result<()> {
    let cmdline = pasta.to_cmdline(child);
    Command::new(cmdline[0].clone())
        .args(&cmdline[1..])
        .output()
        .map_err(ProcessErrorKind::StdIoError)?;
    Ok(())
}
