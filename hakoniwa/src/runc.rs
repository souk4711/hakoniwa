mod error;
mod nix;
mod rlimit;
mod timeout;
mod unshare;

use os_pipe::{PipeReader, PipeWriter};
use std::collections::HashMap;
use std::ffi::CString;
use std::io::prelude::*;
use std::process;
use std::time::{Duration, Instant};

use crate::runc::error::*;
use crate::runc::nix::{ForkResult, Pid, Signal, UsageWho, WaitStatus};
use crate::{Command, Container, ExitStatus, Rusage};

#[cfg(feature = "landlock")]
mod landlock;

#[cfg(feature = "seccomp")]
mod seccomp;

macro_rules! process_exit {
    ($err:ident) => {{
        let err = format!("hakoniwa: {}\n", $err);
        _ = nix::write_stderr(err.as_bytes());
        process::exit(ExitStatus::FAILURE)
    }};
}

pub(crate) const FIN: u8 = 0;
pub(crate) const SETUP_NETWORK: u8 = 1;
pub(crate) const SETUP_UGIDMAP: u8 = 1 << 1;

pub(crate) fn exec(
    command: &Command,
    container: &Container,
    mut stdin: Option<PipeReader>,
    mut stdout: Option<PipeWriter>,
    mut stderr: Option<PipeWriter>,
    mut reader: PipeReader,
    mut writer: PipeWriter,
) {
    let status = match exec_imp(
        command,
        container,
        &mut stdin,
        &mut stdout,
        &mut stderr,
        &mut reader,
        &mut writer,
    ) {
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

    match writer.write_all(&[FIN]) {
        Ok(_) => {}
        Err(err) => process_exit!(err),
    };
    match writer.write_all(&encoded) {
        Ok(_) => {}
        Err(err) => process_exit!(err),
    };
    drop(writer);

    process::exit(status.code);
}

fn exec_imp(
    command: &Command,
    container: &Container,
    stdin: &mut Option<PipeReader>,
    stdout: &mut Option<PipeWriter>,
    stderr: &mut Option<PipeWriter>,
    reader: &mut PipeReader,
    writer: &mut PipeWriter,
) -> Result<ExitStatus> {
    // Redirect standard I/O stream.
    if let Some(stdin) = stdin.take() {
        nix::dup2_stdin(&stdin)?;
        drop(stdin);
    }
    if let Some(stdout) = stdout.take() {
        nix::dup2_stdout(&stdout)?;
        drop(stdout);
    }
    if let Some(stderr) = stderr.take() {
        nix::dup2_stderr(&stderr)?;
        drop(stderr);
    }

    // Die with parent.
    nix::set_pdeathsig(Signal::SIGKILL)?;

    // Unshare namespaces, mount rootfs, etc.
    unshare::unshare(container)?;

    // Notify the main process to setup network/[ug]idmap.
    let operations = container.get_mainp_setup_operations();
    if operations != 0 {
        let mut response = [0];
        writer.write_all(&[operations])?;
        reader.read_exact(&mut response)?;
        match response[0] {
            0 => {}
            SETUP_NETWORK => Err(Error::SetupNetworkFailed)?,
            SETUP_UGIDMAP => Err(Error::SetupUGidmapFailed)?,
            _ => unreachable!(),
        }
    }

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
            format!(
                "process({}) exited with code {}",
                command.get_program(),
                exit_status
            ),
            Some(exit_status),
        ),
        WaitStatus::Signaled(_, signal, _) => (
            128 + signal as i32,
            format!(
                "process({}) received signal {}",
                command.get_program(),
                signal
            ),
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

    // Mount new proc, etc.
    unshare::tidyup(container)?;

    // Switch to the working directory.
    if let Some(dir) = command.get_current_dir() {
        nix::chdir(dir)?
    };

    // Reset SIGPIPE to SIG_DFL
    nix::reset_sigpipe()?;

    // Set resource limit.
    rlimit::setrlimit(container)?;

    // Restrict ambient rights (e.g. global filesystem access).
    #[cfg(feature = "landlock")]
    landlock::load(container)?;

    // Restrict syscalls.
    #[cfg(feature = "seccomp")]
    seccomp::load(container)?;
    #[cfg(not(feature = "seccomp"))]
    nix::set_no_new_privs()?;

    // Execve.
    let program = command.get_program();
    let args = command.get_args();
    let envs = command.get_envs();
    spawn_imp(program, &args, &envs)
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
