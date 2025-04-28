mod completion;
mod run;

use clap::{Parser, Subcommand};
use clap_verbosity_flag::{InfoLevel, Verbosity};

use crate::contrib;

#[derive(Parser)]
#[command(name = "hakoniwa")]
#[command(version, about, long_about = None, disable_help_subcommand = true, styles = contrib::clap::styles())]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[command(flatten, next_display_order = 100)]
    verbose: Verbosity<InfoLevel>,
}

#[allow(clippy::large_enum_variant)]
#[derive(Subcommand)]
enum Commands {
    /// Generate shell autocompletions
    #[clap(hide = true)]
    Completion(completion::CompletionCommand),

    /// Run a COMMAND in a container
    Run(run::RunCommand),
}

pub fn execute() -> i32 {
    let cli = Cli::parse();

    let level_filter = cli.verbose.log_level_filter();
    let debugging = level_filter >= log::LevelFilter::Debug;
    let timestamp: Option<env_logger::fmt::TimestampPrecision> = if debugging {
        Some(env_logger::fmt::TimestampPrecision::Seconds)
    } else {
        None
    };
    env_logger::builder()
        .format_level(debugging)
        .format_target(false)
        .format_timestamp(timestamp)
        .filter_level(level_filter)
        .init();

    let r = match &cli.command {
        Commands::Completion(cmd) => cmd.execute(),
        Commands::Run(cmd) => cmd.execute(),
    };

    if let Err(err) = r {
        log::error!("{}", err);
        1
    } else {
        r.unwrap()
    }
}
