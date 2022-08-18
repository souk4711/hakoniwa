use clap::{AppSettings, Parser, Subcommand};

use crate::cli::run::RunCommand;

#[allow(clippy::large_enum_variant)]
#[derive(Subcommand)]
enum Commands {
    /// Run a COMMAND in a container
    Run(RunCommand),
}

#[derive(Parser)]
#[clap(name = "hakoniwa", version, about, long_about = None)]
#[clap(global_setting(AppSettings::DeriveDisplayOrder))]
pub struct RootCommand {
    #[clap(subcommand)]
    command: Commands,
}

pub fn execute() {
    let cli = RootCommand::parse();
    match &cli.command {
        Commands::Run(cmd) => RunCommand::execute(cmd),
    }
}
