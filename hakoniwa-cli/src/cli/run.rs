use anyhow::Result;
use clap::Args;
use nix::unistd::{Gid, Uid};
use std::env;
use std::sync::LazyLock;

use crate::contrib;
use hakoniwa::{Container, Namespace, Rlimit};

static ENV_SHELL: LazyLock<String> =
    LazyLock::new(|| env::var("SHELL").unwrap_or_else(|_| String::from("/bin/sh")));

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

    /// Mount new tmpfs on CONTAINER_PATH
    #[clap(long, value_name = "CONTAINER_PATH")]
    tmpfsmount: Vec<String>,

    /// Custom hostname in the container (implies --unshare-uts)
    #[clap(long)]
    hostname: Option<String>,

    /// Custom UID in the container
    #[clap(long, value_name = "UID", default_value_t = Uid::current().as_raw())]
    uidmap: u32,

    /// Custom GID in the container
    #[clap(long, value_name = "GID", default_value_t = Gid::current().as_raw())]
    gidmap: u32,

    /// Set an environment variable
    #[clap(long, value_name="NAME=VALUE", value_parser = contrib::clap::parse_key_val_equal::<String, String>)]
    setenv: Vec<(String, String)>,

    /// Bind mount the HOST_PATH on "/hako" with read-write access, then run COMMAND in "/hako"
    #[clap(long, value_name = "HOST_PATH")]
    workdir: Option<String>,

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

    #[clap(value_name = "COMMAND", default_value = &**ENV_SHELL, raw = true)]
    argv: Vec<String>,
}

impl RunCommand {
    pub(crate) fn execute(&self) -> Result<i32> {
        let mut container = Container::new();

        // ARG: --unshare-network
        if contrib::clap::contains_flag("--unshare-network") {
            container.unshare(Namespace::Network);
        }

        // ARG: --unshare-uts
        if contrib::clap::contains_flag("--unshare-uts") {
            container.unshare(Namespace::Uts);
        }

        // ARG: --rootfs
        self.rootfs.as_ref().map(|rootfs| container.rootfs(rootfs));

        // ARG: --bindmount
        for (host_path, container_path) in self.bindmount.iter() {
            container.bindmount(host_path, container_path);
        }

        // ARG: --bindmount-ro
        for (host_path, container_path) in self.bindmount_ro.iter() {
            container.bindmount_ro(host_path, container_path);
        }

        // ARG: --tmpfsmount
        for container_path in self.tmpfsmount.iter() {
            container.tmpfsmount(container_path);
        }

        // ARG: --hostname
        if let Some(hostname) = &self.hostname {
            container.unshare(Namespace::Uts).hostname(hostname);
        }

        // ARG: --uidmap, --gidmap
        container.uidmap(self.uidmap);
        container.gidmap(self.gidmap);

        // ARG: --workdir
        let workdir = if let Some(workdir) = &self.workdir {
            if let Some(dir) = workdir.strip_prefix(":") {
                Some(dir)
            } else {
                container.bindmount(workdir, "/hako");
                Some("/hako")
            }
        } else {
            None
        };

        // ARG: --limit-as, --limit-core, --limit-cpu, --limit-fsize, --limit-nofile
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

        // ARG: -- <COMMAND>...
        let (prog, argv) = (&self.argv[0], &self.argv[1..]);
        let mut command = if prog.starts_with("/") {
            container.command(prog)
        } else {
            let prog_abspath = contrib::pathsearch::find_executable_path(prog);
            container.command(&prog_abspath.unwrap_or(prog.into()).to_string_lossy())
        };

        // ARG: -- <COMMAND>...
        command.args(argv);

        // ARG: --setenv
        for (name, value) in self.setenv.iter() {
            command.env(name, value);
        }

        // ARG: --workdir
        workdir.map(|dir| command.current_dir(dir));

        // ARG: --limit-walltime
        self.limit_walltime.map(|val| command.wait_timeout(val));

        // Execute
        let status = command.status()?;
        Ok(status.code)
    }
}
