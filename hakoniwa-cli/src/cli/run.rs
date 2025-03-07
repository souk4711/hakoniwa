use anyhow::{anyhow, Result};
use clap::Args;
use nix::unistd::{Gid, Uid};
use std::{fs, path::Path, str};

use crate::{config, contrib, seccomp};
use hakoniwa::{Container, Namespace, Rlimit, Runctl};

#[derive(Args)]
pub(crate) struct RunCommand {
    /// Create new CGROUP namespace
    #[clap(long)]
    unshare_cgroup: bool,

    /// Create new IPC namespace
    #[clap(long)]
    unshare_ipc: bool,

    /// Create new NETWORK namespace
    #[clap(long)]
    unshare_network: bool,

    /// Create new UTS namespace
    #[clap(long)]
    unshare_uts: bool,

    /// Use HOST_PATH as the mount point for the container root fs
    #[clap(long, value_name = "HOST_PATH")]
    rootdir: Option<String>,

    /// Bind mount all necessary subdirectories in ROOTFS to the container root with read-only access
    #[clap(long, default_value = "/")]
    rootfs: Option<String>,

    /// Bind mount the HOST_PATH on CONTAINER_PATH with read-only access
    #[clap(short, long, value_name="HOST_PATH:CONTAINER_PATH", value_parser = contrib::clap::parse_bindmount::<String, String>)]
    bindmount_ro: Vec<(String, String)>,

    /// Bind mount the HOST_PATH on CONTAINER_PATH with read-write access
    #[clap(short = 'B', long, value_name="HOST_PATH:CONTAINER_PATH", value_parser = contrib::clap::parse_bindmount::<String, String>)]
    bindmount_rw: Vec<(String, String)>,

    /// Mount new devfs on CONTAINER_PATH
    #[clap(long, value_name = "CONTAINER_PATH")]
    devfs: Vec<String>,

    /// Mount new tmpfs on CONTAINER_PATH
    #[clap(long, value_name = "CONTAINER_PATH")]
    tmpfs: Vec<String>,

    /// Custom UID in the container
    #[clap(short, long, value_name = "UID", default_value_t = Uid::current().as_raw())]
    uidmap: u32,

    /// Custom GID in the container
    #[clap(short, long, value_name = "GID", default_value_t = Gid::current().as_raw())]
    gidmap: u32,

    /// Custom hostname in the container (implies --unshare-uts)
    #[clap(long)]
    hostname: Option<String>,

    /// Set an environment variable
    #[clap(short = 'e', long, value_name="NAME=VALUE", value_parser = contrib::clap::parse_setenv::<String, String>)]
    setenv: Vec<(String, String)>,

    /// Bind mount the HOST_PATH on "/hako" with read-write access, then run COMMAND in "/hako"
    #[clap(short, long, value_name = "HOST_PATH:/hako")]
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

    /// Set seccomp security profile
    #[clap(long, default_value = "podman")]
    seccomp: Option<String>,

    /// Load configuration from a specified file, ignoring all other cli arguments
    #[clap(short, long)]
    config: Option<String>,

    #[clap(value_name = "COMMAND", default_value = "/bin/sh", raw = true)]
    argv: Vec<String>,
}

impl RunCommand {
    pub(crate) fn execute(&self) -> Result<i32> {
        match &self.config {
            Some(c) => self.execute_cfg(c),
            None => self.execute_args(),
        }
    }

    pub(crate) fn execute_cfg(&self, cfg: &str) -> Result<i32> {
        let mut container = Container::new();
        container.runctl(Runctl::MountFallback);

        // ARG: --config
        let data = fs::read_to_string(cfg)
            .map_err(|_| anyhow!("--config: failed to load file {:?} ", cfg))?;
        let cfg = config::load_str(&data).map_err(|e| anyhow!("--config: {}", e))?;

        // CFG: namespaces
        for namespace in cfg.namespaces {
            match namespace.nstype.as_ref() {
                "cgroup" => container.unshare(Namespace::Cgroup),
                "ipc" => container.unshare(Namespace::Ipc),
                "network" => container.unshare(Namespace::Network),
                "uts" => container.unshare(Namespace::Uts),
                _ => Err(anyhow!("TODO:"))?,
            };
        }

        // CFG: mounts
        for mount in cfg.mounts {
            let host_path = &mount.source;
            let container_path = &mount.destination;

            if mount.fstype == "devfs" {
                container.devfsmount(container_path);
                continue;
            }

            if mount.fstype == "tmpfs" {
                container.tmpfsmount(container_path);
                continue;
            };

            if mount.rw {
                fs::canonicalize(host_path)
                    .map_err(|_| anyhow!("--config: path {:?} does not exist", host_path))
                    .map(|host_path| {
                        container.bindmount_rw(&host_path.to_string_lossy(), container_path)
                    })?;
            } else {
                fs::canonicalize(host_path)
                    .map_err(|_| anyhow!("--config: path {:?} does not exist", host_path))
                    .map(|host_path| {
                        container.bindmount_ro(&host_path.to_string_lossy(), container_path)
                    })?;
            }
        }

        // CFG: command
        let mut command = container.command("sh");

        // Execute
        let status = command.status()?;
        if status.exit_code.is_none() {
            // - the Container itself fails
            // - or the Command killed by signal
            log::error!("hakoniwa: {}", format!("{}", status.reason));
        }
        Ok(status.code)
    }

    pub(crate) fn execute_args(&self) -> Result<i32> {
        let mut container = Container::new();
        container.runctl(Runctl::MountFallback);

        // ARG: --unshare-cgroup
        if contrib::clap::contains_flag("--unshare-cgroup") {
            container.unshare(Namespace::Cgroup);
        }

        // ARG: --unshare-ipc
        if contrib::clap::contains_flag("--unshare-ipc") {
            container.unshare(Namespace::Ipc);
        }

        // ARG: --unshare-network
        if contrib::clap::contains_flag("--unshare-network") {
            container.unshare(Namespace::Network);
        }

        // ARG: --unshare-uts
        if contrib::clap::contains_flag("--unshare-uts") {
            container.unshare(Namespace::Uts);
        }

        // ARG: --rootdir
        if let Some(rootdir) = &self.rootdir {
            fs::canonicalize(rootdir)
                .map_err(|_| anyhow!("--rootdir: path {:?} does not exist", rootdir))
                .map(|rootdir| container.rootdir(&rootdir))?;
        };

        // ARG: --rootfs
        if let Some(rootfs) = &self.rootfs {
            fs::canonicalize(rootfs)
                .map_err(|_| anyhow!("--rootfs: path {:?} does not exist", rootfs))
                .map(|rootfs| container.rootfs(&rootfs))?;
        };

        // ARG: --bindmount-ro
        for (host_path, container_path) in self.bindmount_ro.iter() {
            fs::canonicalize(host_path)
                .map_err(|_| anyhow!("--bindmount-ro: path {:?} does not exist", host_path))
                .map(|host_path| {
                    container.bindmount_ro(&host_path.to_string_lossy(), container_path)
                })?;
        }

        // ARG: --bindmount-rw
        for (host_path, container_path) in self.bindmount_rw.iter() {
            fs::canonicalize(host_path)
                .map_err(|_| anyhow!("--bindmount-rw: path {:?} does not exist", host_path))
                .map(|host_path| {
                    container.bindmount_rw(&host_path.to_string_lossy(), container_path)
                })?;
        }

        // ARG: --devfs
        for container_path in self.devfs.iter() {
            container.devfsmount(container_path);
        }

        // ARG: --tmpfs
        for container_path in self.tmpfs.iter() {
            container.tmpfsmount(container_path);
        }

        // ARG: --uidmap, --gidmap
        container.uidmap(self.uidmap);
        container.gidmap(self.gidmap);

        // ARG: --hostname
        if let Some(hostname) = &self.hostname {
            container.unshare(Namespace::Uts).hostname(hostname);
        }

        // ARG: --workdir
        let workdir = if let Some(workdir) = &self.workdir {
            if let Some(dir) = workdir.strip_prefix(":") {
                Some(dir)
            } else {
                fs::canonicalize(workdir)
                    .map_err(|_| anyhow!("--workdir: path {:?} does not exist", workdir))
                    .map(|workdir| container.bindmount_rw(&workdir.to_string_lossy(), "/hako"))?;
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

        // ARG: --seccomp
        let seccomp = &self.seccomp.clone().expect("--seccomp: missing value");
        match seccomp.as_ref() {
            "unconfined" => {}
            "podman" => {
                seccomp::load(seccomp)
                    .map_err(|e| anyhow!("--seccomp: {}", e))
                    .map(|f| container.seccomp_filter(f))?;
            }
            _ => {
                let data = fs::read_to_string(seccomp)
                    .map_err(|_| anyhow!("--seccomp: failed to load file {:?} ", seccomp))?;
                seccomp::load_str(&data)
                    .map_err(|e| anyhow!("--seccomp: {}", e))
                    .map(|f| container.seccomp_filter(f))?;
            }
        }

        // ARG: -- <COMMAND>...
        let (prog, argv) = (&self.argv[0], &self.argv[1..]);
        let mut command = if Path::new(prog).is_absolute() {
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
        if status.exit_code.is_none() {
            // - the Container itself fails
            // - or the Command killed by signal
            log::error!("hakoniwa: {}", format!("{}", status.reason));
        }
        Ok(status.code)
    }
}
