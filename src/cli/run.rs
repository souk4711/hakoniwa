use clap::{Args, ValueHint};
use std::path::PathBuf;

use crate::cli::RootCommand;
use crate::Sandbox;

#[derive(Args)]
pub struct RunCommand {
    /// Bind mount the host path SRC on DEST
    #[clap(long, value_names = &["SRC", "DEST"], value_hint = ValueHint::DirPath)]
    bind: Option<Vec<String>>,

    /// Bind mount the host path SRC readonly on DEST
    #[clap(long, value_names = &["SRC", "DEST"], value_hint = ValueHint::DirPath)]
    ro_bind: Option<Vec<String>>,

    /// Run COMMAND under the specified directory
    #[clap(short, long, parse(from_os_str), default_value =".", value_hint = ValueHint::DirPath)]
    work_dir: PathBuf,

    #[clap(value_name = "COMMAND", default_value = "/bin/sh", raw = true)]
    argv: Vec<String>,
}

impl RunCommand {
    pub fn execute(_cli: &RootCommand, cmd: &RunCommand) {
        let sandbox = Sandbox::new();
        let (prog, argv) = (&cmd.argv[0], &cmd.argv[..]);
        let mut executor = sandbox.command(prog, argv);

        // Arg: bind.
        cmd.bind.iter().for_each(|b| {
            executor.bind(&b[0], &b[1]);
        });

        // Arg: ro-bind.
        cmd.ro_bind.iter().for_each(|b| {
            executor.ro_bind(&b[0], &b[1]);
        });

        // Arg: work-dir.
        executor.current_dir(&cmd.work_dir);

        // Run.
        executor.run();
    }
}
