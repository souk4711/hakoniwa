use clap::Args;
use lazy_static::lazy_static;
use serde_json;
use std::{
    env,
    fs::{self, File},
    io::Write,
    path::PathBuf,
    process,
    string::String,
};

use crate::{cli::RootCommand, contrib, Executor, ExecutorResultStatus, Sandbox, SandboxPolicy};

lazy_static! {
    static ref ENV_SHELL: String = env::var("SHELL").unwrap_or_else(|_| String::from("/bin/sh"));
}

#[derive(Args)]
pub struct RunCommand {
    /// Retain the NETWORK namespace
    #[clap(long)]
    share_net: bool,

    /// Custom UID in the sandbox
    #[clap(long)]
    uid: Option<u32>,

    /// Custom GID in the sandbox
    #[clap(long)]
    gid: Option<u32>,

    /// Custom HOSTNAME in the sandbox
    #[clap(long)]
    hostname: Option<String>,

    /// Limit the maximum size of the COMMAND's virtual memory (address space)
    #[clap(long)]
    limit_as: Option<u64>,

    /// Limit the maximum size of a core file in bytes that the COMMAND may dump
    #[clap(long)]
    limit_core: Option<u64>,

    /// Limit the amount of CPU time that the COMMAND can consume, in seconds
    #[clap(long)]
    limit_cpu: Option<u64>,

    /// Limit the maximum size in bytes of files that the COMMAND may create
    #[clap(long)]
    limit_fsize: Option<u64>,

    /// Limit the maximum file descriptor number that can be opened by the COMMAND
    #[clap(long)]
    limit_nofile: Option<u64>,

    /// Set an environment variable
    #[clap(long, value_name="NAME=VALUE", value_parser = contrib::clap::parse_key_val_equal::<String, String>)]
    setenv: Vec<(String, String)>,

    /// Bind mount the HOST_DIR on CONTAINER_DIR
    #[clap(long, value_name="HOST_DIR:CONTAINER_DIR", value_parser = contrib::clap::parse_key_val_colon::<String, String>)]
    bind: Vec<(String, String)>,

    /// Bind mount the HOST_DIR readonly on CONTAINER_DIR
    #[clap(long, value_name="HOST_DIR:CONTAINER_DIR", value_parser = contrib::clap::parse_key_val_colon::<String, String>)]
    ro_bind: Vec<(String, String)>,

    /// Use the specified policy configuration file [default: KISS-policy.toml]
    #[clap(long)]
    policy_file: Option<PathBuf>,

    /// Generate a JSON-formatted report at the specified location
    #[clap(long)]
    report_file: Option<PathBuf>,

    /// Run COMMAND under the specified directory
    #[clap(short, long, default_value = ".", hide_default_value(true))]
    work_dir: PathBuf,

    #[clap(value_name = "COMMAND", default_value = &ENV_SHELL, raw = true)]
    argv: Vec<String>,
}

impl RunCommand {
    pub fn execute(_cli: &RootCommand, cmd: &RunCommand) {
        let sandbox = {
            let mut sandbox = Sandbox::new();

            // Arg: policy-file
            let policy = match &cmd.policy_file {
                Some(policy_file) => {
                    SandboxPolicy::from_str(&fs::read_to_string(policy_file).unwrap()).unwrap()
                }
                None => SandboxPolicy::KISS_POLICY(),
            };
            sandbox.with_policy(policy);

            sandbox
        };

        // Arg: argv.
        let (prog, argv) = (&cmd.argv[0], &cmd.argv[..]);
        let mut executor = sandbox.command(prog, argv);

        // Arg: share-net.
        if contrib::clap::contains_flag("--share-net") {
            executor.share_net_ns(true);
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
        let result = executor.run();
        let exit_code = result.exit_code.unwrap_or(Executor::EXITCODE_FAILURE);
        match result.status {
            ExecutorResultStatus::Unknown | ExecutorResultStatus::Ok => {}
            _ => eprintln!("hakoniwa: {}", result.reason),
        };

        // Arg: report-file.
        if let Some(report_file) = &cmd.report_file {
            let mut file = File::create(report_file).unwrap();
            let data = serde_json::to_string(&result).unwrap();
            file.write_all(data.as_bytes()).unwrap();
        }

        // Exit.
        process::exit(exit_code);
    }
}
