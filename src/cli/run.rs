use clap::{Args, ValueHint};
use std::path::PathBuf;

use crate::cli::RootCommand;
use crate::Sandbox;

#[derive(Args)]
pub struct RunCommand {
    /// Run COMMAND under the specified directory
    #[clap(long, parse(from_os_str), default_value =".", value_hint = ValueHint::DirPath)]
    work_dir: PathBuf,

    #[clap(value_name = "COMMAND", default_value = "/bin/sh", raw = true)]
    argv: Vec<String>,
}

impl RunCommand {
    pub fn execute(_cli: &RootCommand, cmd: &RunCommand) {
        let sandbox = Sandbox::new();
        let (prog, argv) = (&cmd.argv[0], &cmd.argv[..]);
        let mut executor = sandbox.command(prog, argv);
        executor.current_dir(&cmd.work_dir);
        executor.run();
    }
}
