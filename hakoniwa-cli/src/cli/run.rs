use anyhow::Result;
use clap::Args;
use lazy_static::lazy_static;
use std::env;

use crate::contrib;
use hakoniwa::{Container, Namespace};

lazy_static! {
    static ref ENV_SHELL: String = env::var("SHELL").unwrap_or_else(|_| String::from("/bin/sh"));
}

#[derive(Args)]
pub(crate) struct RunCommand {
    /// Create new NETWORK namespace
    #[clap(long)]
    unshare_network: bool,

    /// Create new UTS namespace
    #[clap(long)]
    unshare_uts: bool,

    /// Custom UID in the container
    #[clap(short, long, value_name = "UID")]
    uidmap: Option<u32>,

    /// Custom GID in the container
    #[clap(short, long, value_name = "GID")]
    gidmap: Option<u32>,

    /// Custom hostname in the container (implies --unshare-uts)
    #[clap(long)]
    hostname: Option<String>,

    /// Set an environment variable
    #[clap(long, value_name="NAME=VALUE", value_parser = contrib::clap::parse_key_val_equal::<String, String>)]
    setenv: Vec<(String, String)>,

    #[clap(value_name = "COMMAND", default_value = &**ENV_SHELL, raw = true)]
    argv: Vec<String>,
}

impl RunCommand {
    pub(crate) fn execute(&self) -> Result<i32> {
        let mut container = Container::new();

        // Arg: --unshare-network
        if contrib::clap::contains_flag("--unshare-network") {
            container.unshare(Namespace::Network);
        }

        // Arg: --unshare-uts
        if contrib::clap::contains_flag("--unshare-uts") {
            container.unshare(Namespace::Uts);
        }

        // Arg: --uidmap & --gidmap
        self.uidmap.map(|id| container.uidmap(id));
        self.gidmap.map(|id| container.gidmap(id));

        // Arg: --hostname.
        if let Some(hostname) = &self.hostname {
            container.unshare(Namespace::Uts).hostname(hostname);
        }

        // COMMAND
        let (prog, argv) = (&self.argv[0], &self.argv[1..]);
        let mut command = container.rootfs("/").command(prog);
        command.args(argv);

        //
        let status = command.status()?;
        Ok(status.code)
    }
}
