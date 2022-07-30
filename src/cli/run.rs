use clap::Args;

use crate::cli::root::RootCommand;
use crate::sandbox::Sandbox;

#[derive(Args)]
pub struct RunCommand {
    #[clap(value_name = "COMMAND", raw = true)]
    argv: Vec<String>,
}

impl RunCommand {
    pub fn execute(_cli: &RootCommand, cmd: &RunCommand) {
        let sandbox = Sandbox::new();
        let (prog, argv) = (&cmd.argv[0], &cmd.argv[..]);
        sandbox.run(prog, argv);
    }
}
