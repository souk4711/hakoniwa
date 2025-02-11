mod error;
mod nix;
mod unshare;

use std::collections::HashMap;
use std::ffi::CString;
use std::io::prelude::*;
use std::process;
use std::time::{Duration, Instant};

use crate::runc::error::*;
use crate::runc::nix::{ForkResult, Pid, UsageWho, WaitStatus};
use crate::{Command, Container, ExitStatus, Rusage};

macro_rules! process_exit {
    ($err:ident) => {{
        let err = format!("hakoniwa: {}", $err);
        _ = nix::write_stderr(err.as_bytes());
        process::exit(ExitStatus::FAILURE)
    }};
}

pub(crate) fn exec(mut writer: os_pipe::PipeWriter, command: &Command, container: &Container) {
    let status = match exec_imp(command, container) {
        Ok(val) => val,
        Err(_) => ExitStatus {
            code: ExitStatus::FAILURE,
            exit_code: None,
            rusage: None,
        },
    };

    let config = bincode::config::standard();
    let encoded: Vec<u8> = match bincode::serde::encode_to_vec(&status, config) {
        Ok(val) => val,
        Err(err) => process_exit!(err),
    };

    match writer.write_all(&encoded) {
        Ok(val) => val,
        Err(err) => process_exit!(err),
    };
    drop(writer);

    process::exit(status.code);
}

fn exec_imp(command: &Command, container: &Container) -> Result<ExitStatus> {
    // Die with parent.
    nix::prctl_set_pdeathsig(libc::SIGKILL)?;

    // Create new session.
    nix::setsid()?;

    // Unshare namespaces.
    unshare::unshare(container)?;

    // Fork the specified program as a child process rather than running it
    // directly. This is useful when creating a new PID namespace.
    match nix::fork()? {
        ForkResult::Parent { child, .. } => reap(child),
        ForkResult::Child => match spawn(command, container) {
            Ok(_) => unreachable!(),
            Err(err) => process_exit!(err),
        },
    }
}

fn reap(child: Pid) -> Result<ExitStatus> {
    let started_at = Instant::now();
    let (code, exit_code) = match nix::waitpid(child)? {
        WaitStatus::Exited(_, exit_status) => (ExitStatus::SUCCESS, Some(exit_status)),
        WaitStatus::Signaled(_, signal, _) => (128 + signal as i32, None),
        _ => (ExitStatus::FAILURE, None),
    };

    let real_time = started_at.elapsed();
    let rusage = nix::getrusage(UsageWho::RUSAGE_CHILDREN)?;

    let user_time = rusage.user_time();
    let user_time = Duration::new(
        user_time.tv_sec() as u64,
        (user_time.tv_usec() * 1000) as u32,
    );

    let system_time = rusage.system_time();
    let system_time = Duration::new(
        system_time.tv_sec() as u64,
        (system_time.tv_usec() * 1000) as u32,
    );

    let max_rss = rusage.max_rss();

    Ok(ExitStatus {
        code,
        exit_code,
        rusage: Some(Rusage {
            real_time,
            user_time,
            system_time,
            max_rss,
        }),
    })
}

fn spawn(command: &Command, _container: &Container) -> Result<()> {
    // Die with parent.
    nix::prctl_set_pdeathsig(libc::SIGKILL)?;

    // Execve.
    let program = command.get_program();
    let args = command.get_args();
    let envs = command.get_envs();
    spawn_imp(program, args, envs)
}

fn spawn_imp<S: AsRef<str>>(
    program: &str,
    args: &[S],
    envs: &HashMap<String, String>,
) -> Result<()> {
    let prog = CString::new(program)?;

    let mut argv = vec![prog.clone()];
    for arg in args {
        let arg = CString::new(arg.as_ref())?;
        argv.push(arg);
    }

    let mut envp = vec![];
    for (k, v) in envs {
        let env = CString::new(format!("{}={}", k, v))?;
        envp.push(env);
    }

    nix::execve(&prog, &argv, &envp)
}
