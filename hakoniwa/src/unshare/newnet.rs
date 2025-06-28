mod network;
mod pasta;

use nix::unistd::Pid;
use std::process::Command;

use crate::{error::*, Container};

pub use network::Network;
pub use pasta::Pasta;

pub(crate) fn mainp_setup(container: &Container, child: Pid) -> Result<()> {
    let network = &container.network.clone().expect("unreachable!");
    match network {
        Network::Pasta(pasta) => mainp_setup_pasta(pasta, child)?,
    }

    log::debug!("================================");
    Ok(())
}

fn mainp_setup_pasta(pasta: &Pasta, child: nix::unistd::Pid) -> Result<()> {
    let cmdline = pasta.to_cmdline(child);
    log::debug!("Configuring Network: Execve: {cmdline:?}");

    let output = Command::new(cmdline[0].clone())
        .args(&cmdline[1..])
        .output();
    match output {
        Ok(output) if output.status.success() => {
            let output = format!("\n{}", String::from_utf8_lossy(&output.stderr).trim());
            log::debug!("Configuring Network: Output: {output}");
            Ok(())
        }
        Ok(output) => {
            let errmsg = format!("\n{}", String::from_utf8_lossy(&output.stderr).trim());
            Err(ProcessErrorKind::SetupNetworkFailed(errmsg))?
        }
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            let errmsg = format!("command {:?} not found", pasta.prog);
            Err(ProcessErrorKind::SetupNetworkFailed(errmsg))?
        }
        Err(err) => {
            let errmsg = format!("{err}");
            Err(ProcessErrorKind::SetupNetworkFailed(errmsg))?
        }
    }
}
