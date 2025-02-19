use anyhow::Result;
use clap::Args;
use lazy_static::lazy_static;
use std::env;

use crate::contrib;
use hakoniwa::{Container, Namespace, Rlimit};

lazy_static! {
    static ref ENV_SHELL: String = env::var("SHELL").unwrap_or_else(|_| String::from("/bin/sh"));
}

#[derive(Args)]
pub(crate) struct RunCommand {
    /// Create new NETWORK namespace
    #[clap(long)]
    unshare_network: bool,

    /// Create new UTS namespace
    #[clap(long)]
    unshare_uts: bool,

    /// Bind mount all necessary subdirectories in ROOTFS to the container root with read-only access
    #[clap(long, default_value = "/")]
    rootfs: Option<String>,

    /// Bind mount the HOST_PATH on CONTAINER_PATH with read-write access
    #[clap(long, value_name="HOST_PATH:CONTAINER_PATH", value_parser = contrib::clap::parse_key_val_colon_path::<String, String>)]
    bindmount: Vec<(String, String)>,

    /// Bind mount the HOST_PATH on CONTAINER_PATH with read-only access
    #[clap(long, value_name="HOST_PATH:CONTAINER_PATH", value_parser = contrib::clap::parse_key_val_colon_path::<String, String>)]
    bindmount_ro: Vec<(String, String)>,

    /// Custom hostname in the container (implies --unshare-uts)
    #[clap(long)]
    hostname: Option<String>,

    /// Custom UID in the container
    #[clap(long, value_name = "UID")]
    uidmap: Option<u32>,

    /// Custom GID in the container
    #[clap(long, value_name = "GID")]
    gidmap: Option<u32>,

    /// Limit the maximum size of the COMMAND's virtual memory
    #[clap(long, value_name = "LIMIT")]
    limit_as: Option<u64>,

    /// Limit the maximum size of a core file in bytes that the COMMAND may dump
    #[clap(long, value_name = "LIMIT")]
    limit_core: Option<u64>,

    /// Limit the amount of CPU time that the COMMAND can consume, in seconds
    #[clap(long, value_name = "LIMIT")]
    limit_cpu: Option<u64>,

    /// Limit the maximum size in bytes of files that the COMMAND may create
    #[clap(long, value_name = "LIMIT")]
    limit_fsize: Option<u64>,

    /// Limit the maximum file descriptor number that can be opened by the COMMAND
    #[clap(long, value_name = "LIMIT")]
    limit_nofile: Option<u64>,

    /// Limit the amount of wall time that the COMMAND can consume, in seconds
    #[clap(long, value_name = "LIMIT")]
    limit_walltime: Option<u64>,

    /// Set an environment variable
    #[clap(long, value_name="NAME=VALUE", value_parser = contrib::clap::parse_key_val_equal::<String, String>)]
    setenv: Vec<(String, String)>,

    #[clap(value_name = "COMMAND", default_value = &**ENV_SHELL, raw = true)]
    argv: Vec<String>,
}

impl RunCommand {
    pub(crate) fn execute(&self) -> Result<i32> {
        let mut container = Container::new();

        // Arg: --unshare-network
        if contrib::clap::contains_flag("--unshare-network") {
            container.unshare(Namespace::Network);
        }

        // Arg: --unshare-uts
        if contrib::clap::contains_flag("--unshare-uts") {
            container.unshare(Namespace::Uts);
        }

        // Arg: --rootfs
        self.rootfs.as_ref().map(|rootfs| container.rootfs(rootfs));

        // Arg: --bindmount
        for (host_path, container_path) in self.bindmount.iter() {
            container.bindmount(host_path, container_path);
        }

        // Arg: --bindmount-ro
        for (host_path, container_path) in self.bindmount_ro.iter() {
            container.bindmount_ro(host_path, container_path);
        }

        // Arg: --hostname
        if let Some(hostname) = &self.hostname {
            container.unshare(Namespace::Uts).hostname(hostname);
        }

        // Arg: --uidmap, --gidmap
        self.uidmap.map(|id| container.uidmap(id));
        self.gidmap.map(|id| container.gidmap(id));

        // Arg: --limit-as, --limit-core, --limit-cpu, --limit-fsize, --limit-nofile
        self.limit_as
            .map(|val| container.setrlimit(Rlimit::As, val, val));
        self.limit_core
            .map(|val| container.setrlimit(Rlimit::Core, val, val));
        self.limit_cpu
            .map(|val| container.setrlimit(Rlimit::Cpu, val, val));
        self.limit_fsize
            .map(|val| container.setrlimit(Rlimit::Fsize, val, val));
        self.limit_nofile
            .map(|val| container.setrlimit(Rlimit::Nofile, val, val));

        // COMMAND
        let (prog, argv) = (&self.argv[0], &self.argv[1..]);
        let mut command = container.command(prog);
        command.args(argv);

        // Arg: --setenv
        for (name, value) in self.setenv.iter() {
            command.env(name, value);
        }

        // Arg: --limit-walltime
        self.limit_walltime.map(|val| command.wait_timeout(val));

        // Execve
        let status = command.status()?;
        Ok(status.code)
    }
}
