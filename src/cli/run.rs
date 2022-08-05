use clap::{Args, ValueHint};
use std::path::PathBuf;

use crate::Sandbox;
use crate::{cli::contrib, cli::RootCommand};

#[derive(Args)]
pub struct RunCommand {
    ///Retain the NETWORK namespace
    #[clap(long, action)]
    share_net: bool,

    /// Custom UID in the sandbox
    #[clap(long)]
    uid: Option<libc::uid_t>,

    /// Custom GID in the sandbox
    #[clap(long)]
    gid: Option<libc::uid_t>,

    /// Custom HOSTNAME in the sandbox
    #[clap(long, default_value = "hakoniwa")]
    hostname: String,

    /// Set an environment variable
    #[clap(long, value_name="NAME=VALUE", value_parser = contrib::parse_key_val_equal::<String, String>)]
    setenv: Vec<(String, String)>,

    /// Bind mount the HOST_DIR on CONTAINER_DIR
    #[clap(long, value_name="HOST_DIR:CONTAINER_DIR", value_parser = contrib::parse_key_val_colon::<String, String>, value_hint = ValueHint::DirPath)]
    bind: Vec<(String, String)>,

    /// Bind mount the HOST_DIR readonly on CONTAINER_DIR
    #[clap(long, value_name="HOST_DIR:CONTAINER_DIR", value_parser = contrib::parse_key_val_colon::<String, String>, value_hint = ValueHint::DirPath)]
    ro_bind: Vec<(String, String)>,

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

        // Arg: share-net.
        executor.share_net_ns(cmd.share_net);

        // Arg: uid.
        if let Some(id) = cmd.uid {
            executor.uid(id);
        }

        // Arg: gid.
        if let Some(id) = cmd.gid {
            executor.gid(id);
        }

        // Arg: hostname.
        executor.hostname(&cmd.hostname);

        // Arg: setenv.
        cmd.setenv.iter().for_each(|(name, value)| {
            executor.setenv(name, value);
        });

        // Arg: bind.
        cmd.bind.iter().for_each(|(host_path, container_path)| {
            executor.bind(host_path, container_path);
        });

        // Arg: ro-bind.
        cmd.ro_bind.iter().for_each(|(host_path, container_path)| {
            executor.ro_bind(host_path, container_path);
        });

        // Arg: work-dir.
        executor.current_dir(&cmd.work_dir);

        // Run.
        executor.run();
    }
}
