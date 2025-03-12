use anyhow::{anyhow, Result};
use clap::Args;
use std::{fs, path::Path, str};

use crate::{config, contrib, seccomp};
use hakoniwa::{Command, Container, Namespace, Rlimit, Runctl};

const SHELL: &str = "/bin/sh";

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
    #[clap(short, long, value_name = "UID")]
    uidmap: Option<u32>,

    /// Custom GID in the container
    #[clap(short, long, value_name = "GID")]
    gidmap: Option<u32>,

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

    #[clap(value_name = "COMMAND", default_value = SHELL, raw = true)]
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
        let cfg = config::load(cfg).map_err(|e| anyhow!("--config: {}", e))?;

        // CFG: namespaces
        for namespace in cfg.namespaces {
            match namespace.nstype.as_ref() {
                "cgroup" => container.unshare(Namespace::Cgroup),
                "ipc" => container.unshare(Namespace::Ipc),
                "network" => container.unshare(Namespace::Network),
                "uts" => container.unshare(Namespace::Uts),
                ns => Err(anyhow!(format!("--config: unknown namespace {:?}", ns)))?,
            };
        }

        // CFG: mounts
        for mount in cfg.mounts {
            let host_path = &mount.source;
            let container_path = &mount.destination.unwrap_or(host_path.to_string());

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

        // CFG: seccomp
        let seccomp = cfg.seccomp.path.unwrap_or("podman".to_string());
        Self::install_seccomp_filter(&mut container, &seccomp)
            .map_err(|e| anyhow!("--config: seccomp: {}", e))?;

        // ARG: -- <COMMAND>...
        // CFG: command::cmdline
        let (prog, argv) = if contrib::clap::contains_arg_raw() {
            (&self.argv[0], &self.argv[1..])
        } else {
            let argv = &cfg.command.cmdline;
            match argv.len() {
                0 => (&SHELL.to_string(), &argv[..]),
                _ => (&argv[0], &argv[1..]),
            }
        };
        let mut command = Self::build_command(&container, prog, argv);

        // CFG: envs
        for env in cfg.envs {
            let (name, value) = env.unwrap_or_default();
            command.env(&name, &value);
        }

        // CFG: command::cwd
        let workdir = cfg.command.cwd;
        workdir.map(|dir| command.current_dir(dir));

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
        if contrib::clap::contains_arg("--unshare-cgroup") {
            container.unshare(Namespace::Cgroup);
        }

        // ARG: --unshare-ipc
        if contrib::clap::contains_arg("--unshare-ipc") {
            container.unshare(Namespace::Ipc);
        }

        // ARG: --unshare-network
        if contrib::clap::contains_arg("--unshare-network") {
            container.unshare(Namespace::Network);
        }

        // ARG: --unshare-uts
        if contrib::clap::contains_arg("--unshare-uts") {
            container.unshare(Namespace::Uts);
        }

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
        self.uidmap.map(|id| container.uidmap(id));
        self.gidmap.map(|id| container.gidmap(id));

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
        Self::install_seccomp_filter(&mut container, seccomp)
            .map_err(|e| anyhow!("--seccomp: {}", e))?;

        // ARG: -- <COMMAND>...
        let (prog, argv) = (&self.argv[0], &self.argv[1..]);
        let mut command = Self::build_command(&container, prog, argv);

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

    fn install_seccomp_filter(container: &mut Container, seccomp: &str) -> Result<()> {
        match seccomp {
            "unconfined" => {}
            "podman" => {
                seccomp::load(seccomp).map(|f| container.seccomp_filter(f))?;
            }
            _ => {
                let data = fs::read_to_string(seccomp)?;
                seccomp::load_str(&data).map(|f| container.seccomp_filter(f))?;
            }
        }
        Ok(())
    }

    fn build_command(container: &Container, prog: &str, argv: &[String]) -> Command {
        if Path::new(prog).is_absolute() {
            let mut cmd = container.command(prog);
            cmd.args(argv);
            cmd
        } else {
            let prog_abspath = contrib::pathsearch::find_executable_path(prog);
            let prog_abspath = prog_abspath.unwrap_or(prog.into());
            let mut cmd = container.command(&prog_abspath.to_string_lossy());
            cmd.args(argv);
            cmd
        }
    }
}
