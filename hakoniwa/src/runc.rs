mod error;
mod nix;
mod rlimit;
mod timeout;
mod unshare;

use os_pipe::{PipeReader, PipeWriter};
use std::collections::HashMap;
use std::ffi::CString;
use std::io::prelude::*;
use std::os::fd::AsRawFd;
use std::process;
use std::time::{Duration, Instant};

use crate::runc::error::*;
use crate::runc::nix::{ForkResult, Pid, Signal, UsageWho, WaitStatus};
use crate::{Command, Container, ExitStatus, Rusage};

macro_rules! process_exit {
    ($err:ident) => {{
        let err = format!("hakoniwa: {}", $err);
        _ = nix::write_stderr(err.as_bytes());
        process::exit(ExitStatus::FAILURE)
    }};
}

pub(crate) fn exec(
    command: &Command,
    container: &Container,
    mut stdin: Option<PipeReader>,
    mut stdout: Option<PipeWriter>,
    mut stderr: Option<PipeWriter>,
    mut status_writer: PipeWriter,
) {
    let status = match exec_imp(command, container, &mut stdin, &mut stdout, &mut stderr) {
        Ok(val) => val,
        Err(err) => ExitStatus {
            code: ExitStatus::FAILURE,
            reason: err.to_string(),
            exit_code: None,
            rusage: None,
        },
    };

    let config = bincode::config::standard();
    let encoded: Vec<u8> = match bincode::serde::encode_to_vec(&status, config) {
        Ok(val) => val,
        Err(err) => process_exit!(err),
    };

    match status_writer.write_all(&encoded) {
        Ok(val) => val,
        Err(err) => process_exit!(err),
    };
    drop(status_writer);

    process::exit(status.code);
}

fn exec_imp(
    command: &Command,
    container: &Container,
    stdin: &mut Option<PipeReader>,
    stdout: &mut Option<PipeWriter>,
    stderr: &mut Option<PipeWriter>,
) -> Result<ExitStatus> {
    // Redirect standard I/O stream.
    if let Some(stdin) = stdin.take() {
        nix::dup2(stdin.as_raw_fd(), libc::STDIN_FILENO)?;
        drop(stdin);
    }
    if let Some(stdout) = stdout.take() {
        nix::dup2(stdout.as_raw_fd(), libc::STDOUT_FILENO)?;
        drop(stdout);
    }
    if let Some(stderr) = stderr.take() {
        nix::dup2(stderr.as_raw_fd(), libc::STDERR_FILENO)?;
        drop(stderr);
    }

    // Die with parent.
    nix::set_pdeathsig(Signal::SIGKILL)?;

    // Create new session.
    nix::setsid()?;

    // Unshare namespaces, mount rootfs, etc.
    unshare::unshare(container)?;

    // Fork the specified program as a child process rather than running it
    // directly. This is useful when creating a new PID namespace.
    match nix::fork()? {
        ForkResult::Parent { child, .. } => reap(child, command),
        ForkResult::Child => match spawn(command, container) {
            Ok(_) => unreachable!(),
            Err(err) => process_exit!(err),
        },
    }
}

fn reap(child: Pid, command: &Command) -> Result<ExitStatus> {
    if let Some(timeout) = command.wait_timeout {
        timeout::timeout(child, timeout)?;
    }

    let started_at = Instant::now();
    let (code, reason, exit_code) = match nix::waitpid(child)? {
        WaitStatus::Exited(_, exit_status) => (
            exit_status,
            format!("waitpid(...) => Exited(_, {})", exit_status),
            Some(exit_status),
        ),
        WaitStatus::Signaled(_, signal, _) => (
            128 + signal as i32,
            format!("waitpid(...) => Signaled(_, {}, _)", signal),
            None,
        ),
        ws => (
            ExitStatus::FAILURE,
            format!("waitpid(...) => {:?}", ws),
            None,
        ),
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
        reason,
        exit_code,
        rusage: Some(Rusage {
            real_time,
            user_time,
            system_time,
            max_rss,
        }),
    })
}

fn spawn(command: &Command, container: &Container) -> Result<()> {
    // Die with parent.
    nix::set_pdeathsig(Signal::SIGKILL)?;

    // Remount rootfs, etc.
    unshare::tidyup(container)?;

    // Set resource limit.
    rlimit::setrlimit(container)?;

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
