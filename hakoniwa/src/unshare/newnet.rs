mod network;
mod pasta;

use nix::unistd::Pid;
use std::process::Command;

use crate::{error::*, Container};

pub use network::Network;
pub use pasta::Pasta;

pub(crate) fn mainp_setup(container: &Container, child: Pid) -> Result<()> {
    match &container.network {
        Some(network) => match network {
            Network::Pasta(pasta) => mainp_setup_pasta(pasta, child)?,
        },
        None => unreachable!(),
    };

    log::debug!("================================");
    Ok(())
}

fn mainp_setup_pasta(pasta: &Pasta, child: nix::unistd::Pid) -> Result<()> {
    let cmdline = pasta.to_cmdline(child);
    log::debug!("Configuring Network: Execve: \n{:?}", cmdline);

    let output = Command::new(cmdline[0].clone())
        .args(&cmdline[1..])
        .output();
    let output = match output {
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            log::debug!("Configuring Network: Output: \n{}", &stderr);
            output
        }
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            let errmsg = format!("Command {:?} not found", pasta.prog);
            log::debug!("Configuring Network: Output: \n{}", errmsg);
            Err(ProcessErrorKind::StdIoError(err))?
        }
        Err(err) => {
            log::debug!("Configuring Network: Output: \n{}", err);
            Err(ProcessErrorKind::StdIoError(err))?
        }
    };

    if output.status.success() {
        Ok(())
    } else {
        Err(ProcessErrorKind::SetupNetworkFailed)?
    }
}
