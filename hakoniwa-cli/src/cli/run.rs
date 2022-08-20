use clap::Args;
use lazy_static::lazy_static;
use std::{
    env,
    fs::{self, File},
    io::Write,
    path::PathBuf,
    process, str,
    string::String,
};

use crate::{contrib, Embed, Error, Result};
use hakoniwa::{Executor, Sandbox, SandboxPolicy, Stdio};

lazy_static! {
    static ref ENV_SHELL: String = env::var("SHELL").unwrap_or_else(|_| String::from("/bin/sh"));
}

#[derive(Args)]
pub struct RunCommand {
    /// Retain the NETWORK namespace
    #[clap(long)]
    share_net: bool,

    /// Retain the UTS namespace
    #[clap(long)]
    share_uts: bool,

    /// Custom UID in the container
    #[clap(short, long)]
    uid: Option<u32>,

    /// Custom GID in the container
    #[clap(short, long)]
    gid: Option<u32>,

    /// Custom HOSTNAME in the container
    #[clap(long)]
    hostname: Option<String>,

    /// Mount a new tmpfs under "/tmp" in the container
    #[clap(long)]
    mount_new_tmpfs: bool,

    /// Mount a new devfs under "/dev" in the container
    #[clap(long)]
    mount_new_devfs: bool,

    /// Bind mount the HOST_PATH on CONTAINER_PATH with read-only access
    #[clap(long, value_name="HOST_PATH:CONTAINER_PATH", value_parser = contrib::clap::parse_key_val_colon_path::<String, String>)]
    ro_bind: Vec<(String, String)>,

    /// Bind mount the HOST_PATH on CONTAINER_PATH with read-write access
    #[clap(long, value_name="HOST_PATH:CONTAINER_PATH", value_parser = contrib::clap::parse_key_val_colon_path::<String, String>)]
    rw_bind: Vec<(String, String)>,

    /// Bind mount the HOST_PATH on '/hako' with read-write access, then run COMMAND
    #[clap(short, long, value_name = "HOST_PATH")]
    work_dir: Option<PathBuf>,

    /// Set an environment variable
    #[clap(long, value_name="NAME=VALUE", value_parser = contrib::clap::parse_key_val_equal::<String, String>)]
    setenv: Vec<(String, String)>,

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

    /// Use the specified policy configuration file [default: KISS-policy.toml]
    #[clap(long, value_name = "FILE")]
    policy_file: Option<PathBuf>,

    /// Generate a JSON-formatted report at the specified location
    #[clap(long, value_name = "FILE")]
    report_file: Option<PathBuf>,

    /// Use verbose output
    #[clap(short, long, action)]
    verbose: bool,

    #[clap(value_name = "COMMAND", default_value = &ENV_SHELL, raw = true)]
    argv: Vec<String>,
}

impl RunCommand {
    pub fn execute(cmd: &Self) {
        if cmd.verbose {
            env_logger::Builder::new()
                .filter(Some("hakoniwa"), log::LevelFilter::Info)
                .init();
        }

        if let Err(err) = Self::_execute(cmd) {
            eprintln!("hakoniwa-run: {}", err);
            process::exit(Executor::EXITCODE_FAILURE);
        }
    }

    fn _execute(cmd: &Self) -> Result<()> {
        let mut sandbox = Sandbox::new();

        // Arg: policy-file.
        let policy_data = match &cmd.policy_file {
            Some(policy_file) => {
                log::info!(target: "hakoniwa::cli::run", "Configuration: {:?}", policy_file);
                fs::read_to_string(policy_file).unwrap()
            }
            None => {
                log::info!(target: "hakoniwa::cli::run", "Configuration: {:?}", "KISS-policy.toml");
                let f = Embed::get("KISS-policy.toml").unwrap();
                str::from_utf8(&f.data).unwrap().to_string()
            }
        };
        let policy = SandboxPolicy::from_str(&policy_data)?;
        sandbox.with_policy(policy);

        // Arg: argv.
        let (prog, argv) = (&cmd.argv[0], &cmd.argv[..]);
        let mut executor = sandbox.command(prog, argv);

        // Arg: share-net.
        if contrib::clap::contains_flag("--share-net") {
            executor.share_net_ns(true);
        }

        // Arg: share-uts.
        if contrib::clap::contains_flag("--share-uts") {
            executor.share_uts_ns(true);
        }

        // Arg: uid.
        if let Some(id) = cmd.uid {
            executor.uid(id);
        }

        // Arg: gid.
        if let Some(id) = cmd.gid {
            executor.gid(id);
        }

        // Arg: hostname.
        if let Some(hostname) = &cmd.hostname {
            executor.hostname(hostname);
        }

        // Arg: mount-new-tmpfs.
        if contrib::clap::contains_flag("--mount-new-tmpfs") {
            executor.mount_new_tmpfs(true);
        }

        // Arg: mount-new-devfs.
        if contrib::clap::contains_flag("--mount-new-devfs") {
            executor.mount_new_devfs(true);
        }

        // Arg: limit-as.
        if let Some(limit_as) = cmd.limit_as {
            executor.limit_as(Some(limit_as));
        }

        // Arg: limit-core.
        if let Some(limit_core) = cmd.limit_core {
            executor.limit_core(Some(limit_core));
        }

        // Arg: limit-cpu.
        if let Some(limit_cpu) = cmd.limit_cpu {
            executor.limit_cpu(Some(limit_cpu));
        }

        // Arg: limit-fsize.
        if let Some(limit_fsize) = cmd.limit_fsize {
            executor.limit_fsize(Some(limit_fsize));
        }

        // Arg: limit-nofile.
        if let Some(limit_nofile) = cmd.limit_nofile {
            executor.limit_nofile(Some(limit_nofile));
        }

        // Arg: limit-walltime.
        if let Some(limit_walltime) = cmd.limit_walltime {
            executor.limit_walltime(Some(limit_walltime));
        }

        // Arg: setenv.
        for (name, value) in cmd.setenv.iter() {
            executor.setenv(name, value);
        }

        // Arg: ro-bind.
        for (host_path, container_path) in cmd.ro_bind.iter() {
            executor.ro_bind(host_path, container_path)?;
        }

        // Arg: rw-bind.
        for (host_path, container_path) in cmd.rw_bind.iter() {
            executor.rw_bind(host_path, container_path)?;
        }

        // Arg: work-dir.
        if let Some(work_dir) = &cmd.work_dir {
            executor.rw_bind(work_dir, "/hako")?.current_dir("/hako")?;
        }

        // Run.
        let result = executor
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .stdin(Stdio::inherit())
            .run();

        // Arg: report-file.
        if let Some(report_file) = &cmd.report_file {
            let map_io_err = |err: std::io::Error| {
                Error::FileIoError(report_file.to_path_buf(), err.to_string())
            };
            let map_serde_err = |err: serde_json::Error| {
                Error::FileIoError(report_file.to_path_buf(), err.to_string())
            };
            let mut file = File::create(report_file).map_err(map_io_err)?;
            let data = serde_json::to_string(&result).map_err(map_serde_err)?;
            file.write_all(data.as_bytes()).map_err(map_io_err)?;
        }

        // Exit.
        let exit_code = result.exit_code.unwrap_or(Executor::EXITCODE_FAILURE);
        process::exit(exit_code);
    }
}
