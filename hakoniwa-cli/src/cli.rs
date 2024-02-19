mod run;

use clap::{AppSettings, Parser, Subcommand};

use crate::cli::run::RunCommand;

#[derive(Parser)]
#[clap(name = "hakoniwa", version, about, long_about = None)]
#[clap(global_setting(AppSettings::DeriveDisplayOrder))]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[allow(clippy::large_enum_variant)]
#[derive(Subcommand)]
enum Commands {
    /// Run a COMMAND in a container
    Run(RunCommand),
}

pub fn execute() {
    let cli = Cli::parse();
    match &cli.command {
        Commands::Run(cmd) => cmd.execute(),
    }
}
