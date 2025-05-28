use anyhow::{anyhow, Result};
use clap::{Args, ValueHint};
use std::fs;
use std::path::Path;
use std::str::{self, FromStr};

use crate::{cli::argparse, config, seccomp, contrib};
use hakoniwa::{landlock::*, Command, Container, Namespace, Pasta, Rlimit, Runctl};

const SHELL: &str = "/bin/sh";

#[derive(Args)]
pub(crate) struct RunCommand {
    /// Create new CGROUP, IPC, NETWORK, UTS, ... namespaces
    #[clap(long)]
    unshare_all: bool,

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

    /// Use ROOTDIR as the mount point for the container root fs
    #[clap(long, value_name="ROOTDIR:OPTIONS", value_parser = argparse::parse_rootdir::<String, String>, value_hint = ValueHint::DirPath)]
    rootdir: Option<(String, String)>,

    /// Bind mount all subdirectories in ROOTFS to the container root with read-only access
    #[clap(long, default_value = "/", value_hint = ValueHint::DirPath)]
    rootfs: Option<String>,

    /// Bind mount the HOST_PATH on CONTAINER_PATH with read-only access (repeatable)
    #[clap(short, long, value_name="HOST_PATH:CONTAINER_PATH", value_parser = argparse::parse_bindmount::<String, String>, value_hint = ValueHint::DirPath)]
    bindmount_ro: Vec<(String, String)>,

    /// Bind mount the HOST_PATH on CONTAINER_PATH with read-write access (repeatable)
    #[clap(short = 'B', long, value_name="HOST_PATH:CONTAINER_PATH", value_parser = argparse::parse_bindmount::<String, String>, value_hint = ValueHint::DirPath)]
    bindmount_rw: Vec<(String, String)>,

    /// Mount new devfs on CONTAINER_PATH (repeatable)
    #[clap(long, value_name = "CONTAINER_PATH")]
    devfs: Vec<String>,

    /// Mount new tmpfs on CONTAINER_PATH (repeatable)
    #[clap(long, value_name = "CONTAINER_PATH")]
    tmpfs: Vec<String>,

    /// Create a new dir on CONTAINER_PATH with 700 permissions (repeatable)
    #[clap(long, value_name = "CONTAINER_PATH", value_hint = ValueHint::DirPath)]
    dir: Vec<String>,

    /// Create a symbolic link on LINK_PATH pointing to the ORIGINAL_PATH (repeatable)
    #[clap(long, value_name = "ORIGINAL_PATH:LINK_PATH", value_parser = argparse::parse_symlink::<String, String>, value_hint = ValueHint::DirPath)]
    symlink: Vec<(String, String)>,

    /// Custom UID in the container
    #[clap(short, long, value_name = "UID")]
    uidmap: Option<u32>,

    /// Custom GID in the container
    #[clap(short, long, value_name = "GID")]
    gidmap: Option<u32>,

    /// Custom hostname in the container (implies --unshare-uts)
    #[clap(long)]
    hostname: Option<String>,

    /// Configure network for the container
    #[clap(long, value_name="MODE:OPTIONS", value_parser = argparse::parse_network::<String, String>)]
    network: Option<(String, String)>,

    /// Set an environment variable (repeatable)
    #[clap(short = 'e', long, value_name="NAME=VALUE", value_parser = argparse::parse_setenv::<String, String>)]
    setenv: Vec<(String, String)>,

    /// Bind mount the HOST_PATH on the same container path with read-write access, then run COMMAND inside it
    #[clap(short, long, value_name = "HOST_PATH", value_hint = ValueHint::DirPath)]
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

    /// Restrict ambient rights (e.g. global filesystem access) for the process
    #[clap(long, value_name = "[RESOURCE, ...]")]
    landlock_restrict: Option<String>,

    /// Allow to read files beneath PATH (implies --landlock-restrict=fs)
    #[clap(long, value_name = "[PATH, ...]")]
    landlock_fs_ro: Option<String>,

    /// Allow to read-write files beneath PATH (implies --landlock-restrict=fs)
    #[clap(long, value_name = "[PATH, ...]")]
    landlock_fs_rw: Option<String>,

    /// Allow to execute files beneath PATH (implies --landlock-restrict=fs)
    #[clap(long, value_name = "[PATH, ...]")]
    landlock_fs_rx: Option<String>,

    /// Allow binding a TCP socket to a local port (implies --landlock-restrict=tcp.bind)
    #[clap(long, value_name = "[PORT, ...]")]
    landlock_tcp_bind: Option<String>,

    /// Allow connecting an active TCP socket to a remote port (implies --landlock-restrict=tcp.connect)
    #[clap(long, value_name = "[PORT, ...]")]
    landlock_tcp_connect: Option<String>,

    /// Set seccomp security profile
    #[clap(long, default_value = "podman", value_hint = ValueHint::FilePath)]
    seccomp: Option<String>,

    /// Load configuration from a specified file, ignoring all other cli arguments
    #[clap(short, long, value_hint = ValueHint::FilePath)]
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
            let ns = Self::str_to_namespace(&namespace.nstype)
                .map_err(|e| anyhow!("--config: namespace: {}", e))?;
            if namespace.share {
                container.share(ns);
            } else {
                container.unshare(ns);
            }
        }

        // CFG: rootdir
        if let Some(rootdir) = cfg.rootdir {
            fs::canonicalize(&rootdir.path)
                .map_err(|_| anyhow!("--config: rootdir: path {:?} does not exist", rootdir.path))
                .map(|path| container.rootdir(&path))?;
            if rootdir.rw {
                container.runctl(Runctl::RootdirRW);
            }
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
                    .map_err(|_| anyhow!("--config: mount: path {:?} does not exist", host_path))
                    .map(|host_path| {
                        container.bindmount_rw(&host_path.to_string_lossy(), container_path)
                    })?;
            } else {
                fs::canonicalize(host_path)
                    .map_err(|_| anyhow!("--config: mount: path {:?} does not exist", host_path))
                    .map(|host_path| {
                        container.bindmount_ro(&host_path.to_string_lossy(), container_path)
                    })?;
            }
        }

        // CFG: filesystem
        if let Some(filesystem) = cfg.filesystem {
            for dir in filesystem.dirs {
                container.dir(&dir.destination, 0o700);
            }

            for symlink in filesystem.symlinks {
                let original = symlink.original;
                let link = symlink.link;
                container.symlink(&original, &link);
            }
        }

        // CFG: uidmap, gidmap
        cfg.uidmap.map(|idmap| container.uidmap(idmap.container_id));
        cfg.gidmap.map(|idmap| container.gidmap(idmap.container_id));

        // CFG: hostname
        let hostname = cfg.hostname;
        hostname.map(|name| container.unshare(Namespace::Uts).hostname(&name));

        // CFG: network
        if let Some(network) = cfg.network {
            Self::configure_network(&mut container, &network.mode, &network.options)
                .map_err(|e| anyhow!("--config: network: {}", e))?;
        }

        // CFG: limits
        let mut limit_walltime = None;
        for limit in cfg.limits {
            if limit.rtype == "walltime" {
                limit_walltime = Some(limit.value);
            } else {
                let lim = Self::str_to_rlimit(&limit.rtype)
                    .map_err(|e| anyhow!("--config: limit: {}", e))?;
                container.setrlimit(lim, limit.value, limit.value);
            }
        }

        // CFG: landlock
        if let Some(landlock) = cfg.landlock {
            let mut ruleset = Ruleset::default();
            for resource in landlock.resources {
                let res = Self::str_to_landlock_resource(&resource.rtype)
                    .map_err(|e| anyhow!("--config: landlock: {}", e))?;
                if resource.unrestrict {
                    ruleset.unrestrict(res);
                } else {
                    ruleset.restrict(res, CompatMode::Enforce);
                }
            }

            for rule in landlock.fs {
                let access = FsAccess::from_str(&rule.access)
                    .map_err(|e| anyhow!("--config: landlock: {}", e))?;
                ruleset.add_fs_rule(&rule.path, access);
            }

            for rule in landlock.net {
                let access = Self::str_to_landlock_net_access(&rule.access)
                    .map_err(|e| anyhow!("--config: landlock: {}", e))?;
                ruleset.add_net_rule(rule.port, access);
            }

            container.landlock_ruleset(ruleset);
        }

        // CFG: seccomp
        let seccomp = cfg.seccomp.path.unwrap_or("podman".to_string());
        Self::install_seccomp_filter(&mut container, &seccomp)
            .map_err(|e| anyhow!("--config: seccomp: {}", e))?;

        // ARG: -- <COMMAND>...
        // CFG: command::cmdline
        let (prog, argv) = if argparse::contains_arg_raw() {
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

        // CFG: limits::walltime
        limit_walltime.map(|val| command.wait_timeout(val));

        // Execute
        let status = command.status()?;
        if status.exit_code.is_none() {
            // - the Container itself fails
            // - or the Command killed by signal
            log::error!("hakoniwa: {}", status.reason);
        }
        Ok(status.code)
    }

    pub(crate) fn execute_args(&self) -> Result<i32> {
        let mut container = Container::new();
        container.runctl(Runctl::MountFallback);

        // ARG: --unshare-all, --unshare-cgroup
        if argparse::contains_arg("--unshare-all") || argparse::contains_arg("--unshare-cgroup") {
            container.unshare(Namespace::Cgroup);
        }

        // ARG: --unshare-all, --unshare-ipc
        if argparse::contains_arg("--unshare-all") || argparse::contains_arg("--unshare-ipc") {
            container.unshare(Namespace::Ipc);
        }

        // ARG: --unshare-all, --unshare-network
        if argparse::contains_arg("--unshare-all") || argparse::contains_arg("--unshare-network") {
            container.unshare(Namespace::Network);
        }

        // ARG: --unshare-all, --unshare-uts
        if argparse::contains_arg("--unshare-all") || argparse::contains_arg("--unshare-uts") {
            container.unshare(Namespace::Uts);
        }

        // ARG: --rootdir
        if let Some((path, options)) = &self.rootdir {
            fs::canonicalize(path)
                .map_err(|_| anyhow!("--rootdir: path {:?} does not exist", path))
                .map(|path| container.rootdir(&path))?;
            if options == "rw" {
                container.runctl(Runctl::RootdirRW);
            }
        };

        // ARG: --rootfs
        if let Some(rootfs) = &self.rootfs {
            if rootfs != "none" {
                fs::canonicalize(rootfs)
                    .map_err(|_| anyhow!("--rootfs: path {:?} does not exist", rootfs))
                    .map(|rootfs| container.rootfs(&rootfs))?;
            }
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

        // ARG: --workdir
        let workdir = if let Some(workdir) = &self.workdir {
            if let Some(dir) = workdir.strip_prefix(":") {
                Some(dir.to_string())
            } else {
                let dir = fs::canonicalize(workdir)
                    .map_err(|_| anyhow!("--workdir: path {:?} does not exist", workdir))?;
                container.bindmount_rw(&dir.to_string_lossy(), &dir.to_string_lossy());
                Some(dir.to_string_lossy().to_string())
            }
        } else {
            None
        };

        // ARG: --dir
        for container_path in self.dir.iter() {
            container.dir(container_path, 0o700);
        }

        // ARG: --symlink
        for (original, link) in self.symlink.iter() {
            container.symlink(original, link);
        }

        // ARG: --uidmap, --gidmap
        self.uidmap.map(|id| container.uidmap(id));
        self.gidmap.map(|id| container.gidmap(id));

        // ARG: --hostname
        if let Some(hostname) = &self.hostname {
            container.unshare(Namespace::Uts).hostname(hostname);
        }

        // ARG: --network
        if let Some((mode, options)) = &self.network {
            let options = argparse::parse_network_options(options)?;
            Self::configure_network(&mut container, mode, &options)
                .map_err(|e| anyhow!("--network: {}", e))?;
        }

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

        // ARG: --landlock
        if argparse::contains_arg_landlock() {
            let mut ruleset = Ruleset::default();

            // ARG: --landlock-restrict
            if let Some(resources) = &self.landlock_restrict {
                for resource in resources.split(&[',']) {
                    let resource = Self::str_to_landlock_resource(resource)
                        .map_err(|e| anyhow!("--landlock-restrict: {}", e))?;
                    ruleset.restrict(resource, CompatMode::Enforce);
                }
            }

            // ARG: --landlock-fs-ro
            if let Some(paths) = &self.landlock_fs_ro {
                ruleset.restrict(Resource::FS, CompatMode::Enforce);
                for path in paths.split(&[',', ':']) {
                    ruleset.add_fs_rule(path, FsAccess::R);
                }
            }

            // ARG: --landlock-fs-rw
            if let Some(paths) = &self.landlock_fs_rw {
                ruleset.restrict(Resource::FS, CompatMode::Enforce);
                for path in paths.split(&[',', ':']) {
                    ruleset.add_fs_rule(path, FsAccess::R | FsAccess::W);
                }
            }

            // ARG: --landlock-fs-rx
            if let Some(paths) = &self.landlock_fs_rx {
                ruleset.restrict(Resource::FS, CompatMode::Enforce);
                for path in paths.split(&[',', ':']) {
                    ruleset.add_fs_rule(path, FsAccess::R | FsAccess::X);
                }
            }

            // ARG: --landlock-tcp-bind
            if let Some(ports) = &self.landlock_tcp_bind {
                ruleset.restrict(Resource::NET_TCP_BIND, CompatMode::Enforce);
                for port in argparse::parse_landlock_net_ports(ports)? {
                    ruleset.add_net_rule(port, NetAccess::TCP_BIND);
                }
            }

            // ARG: --landlock-tcp-connect
            if let Some(ports) = &self.landlock_tcp_connect {
                ruleset.restrict(Resource::NET_TCP_CONNECT, CompatMode::Enforce);
                for port in argparse::parse_landlock_net_ports(ports)? {
                    ruleset.add_net_rule(port, NetAccess::TCP_CONNECT);
                }
            }

            container.landlock_ruleset(ruleset);
        }

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
            log::error!("hakoniwa: {}", status.reason);
        }
        Ok(status.code)
    }

    fn configure_network(container: &mut Container, mode: &str, options: &[String]) -> Result<()> {
        match mode {
            "none" => container.unshare(Namespace::Network),
            "host" => container.share(Namespace::Network),
            "pasta" => {
                let mut pasta = Pasta::default();
                pasta.args(options);
                container.unshare(Namespace::Network).network(pasta)
            }
            _ => {
                let msg = format!("unknown mode {:?}", mode);
                Err(anyhow!(msg))?
            }
        };
        Ok(())
    }

    fn install_seccomp_filter(container: &mut Container, seccomp: &str) -> Result<()> {
        match seccomp {
            "unconfined" => {}
            "audit" | "podman" => {
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

    fn str_to_namespace(s: &str) -> Result<Namespace> {
        Ok(match s {
            "cgroup" => Namespace::Cgroup,
            "ipc" => Namespace::Ipc,
            "network" => Namespace::Network,
            "uts" => Namespace::Uts,
            _ => {
                let msg = format!("unknown namespace type {:?}", s);
                Err(anyhow!(msg))?
            }
        })
    }

    fn str_to_rlimit(s: &str) -> Result<Rlimit> {
        Ok(match s {
            "as" => Rlimit::As,
            "core" => Rlimit::Core,
            "cpu" => Rlimit::Cpu,
            "fsize" => Rlimit::Fsize,
            "nofile" => Rlimit::Nofile,
            _ => {
                let msg = format!("unknown limit type {:?}", s);
                Err(anyhow!(msg))?
            }
        })
    }

    fn str_to_landlock_resource(s: &str) -> Result<Resource> {
        Ok(match s {
            "fs" => Resource::FS,
            "tcp.bind" => Resource::NET_TCP_BIND,
            "tcp.connect" => Resource::NET_TCP_CONNECT,
            _ => {
                let msg = format!("unknown resource type {:?}", s);
                Err(anyhow!(msg))?
            }
        })
    }

    fn str_to_landlock_net_access(s: &str) -> Result<NetAccess> {
        Ok(match s {
            "tcp.bind" => NetAccess::TCP_BIND,
            "tcp.connect" => NetAccess::TCP_CONNECT,
            _ => {
                let msg = format!("unknown net access {:?}", s);
                Err(anyhow!(msg))?
            }
        })
    }
}
