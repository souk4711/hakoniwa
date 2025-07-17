use anyhow::{Result, anyhow};
use clap::{Args, Command, CommandFactory};
use clap_complete::{Generator, Shell};
use std::fs::File;
use std::io::{self, Write};

#[derive(Args)]
pub(crate) struct CompletionCommand {
    /// Output the completion to file rather than stdout
    #[clap(short, long)]
    file: Option<String>,

    #[arg(value_name = "SHELL", value_enum)]
    shell: Shell,
}

impl CompletionCommand {
    pub(crate) fn execute(&self) -> Result<i32> {
        let mut cmd = super::super::Cli::command();

        // ARG: --file
        if let Some(file) = &self.file {
            let mut buf = File::create(file).map_err(|e| anyhow!("--file: {}", e))?;
            Self::print_completions(self.shell, &mut cmd, &mut buf);
        } else {
            let mut buf = io::stdout();
            Self::print_completions(self.shell, &mut cmd, &mut buf);
        }

        Ok(0)
    }

    fn print_completions<G: Generator>(generator: G, cmd: &mut Command, buf: &mut dyn Write) {
        clap_complete::generate(generator, cmd, cmd.get_name().to_string(), buf);
    }
}
