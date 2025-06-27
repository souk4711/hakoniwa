mod argparse;
mod pathsearch;
mod subcommands;

use clap::builder::styling::{AnsiColor, Styles};
use clap::{Parser, Subcommand};
use clap_verbosity_flag::{InfoLevel, Verbosity};

const AFTER_HELP: &str =
    "To view the user documentation, please visit https://github.com/souk4711/hakoniwa.";

fn styles() -> Styles {
    Styles::styled()
        .header(AnsiColor::Yellow.on_default())
        .usage(AnsiColor::Green.on_default())
        .literal(AnsiColor::Green.on_default())
        .placeholder(AnsiColor::Green.on_default())
}

#[derive(Parser)]
#[command(name = "hakoniwa", version)]
#[command(about, long_about = None)]
#[command(disable_help_subcommand = true, after_help = AFTER_HELP)]
#[command(styles = styles())]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[command(flatten, next_display_order = 100)]
    verbose: Verbosity<InfoLevel>,
}

#[allow(clippy::large_enum_variant)]
#[derive(Subcommand)]
enum Commands {
    /// Generate SHELL autocompletions
    Completion(subcommands::CompletionCommand),

    /// Run a COMMAND in a container
    Run(subcommands::RunCommand),
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
        log::error!("{err}");
        1
    } else {
        r.unwrap()
    }
}
