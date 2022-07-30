use clap::Args;

use crate::cli::root::RootCommand;

#[derive(Args)]
pub struct ServerCommand {
    #[clap(short, long, value_parser, value_name = "HOST")]
    bind: Option<String>,

    #[clap(
        short,
        long,
        value_parser = clap::value_parser!(u16).range(1..),
    )]
    port: Option<u16>,
}

impl ServerCommand {
    pub fn execute(_cli: &RootCommand, _cmd: &ServerCommand) {}
}
