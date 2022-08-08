use clap::Args;

use crate::cli::RootCommand;

#[derive(Args)]
pub struct ServerCommand {}

impl ServerCommand {
    pub fn execute(_cli: &RootCommand, _cmd: &Self) {}
}
