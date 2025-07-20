mod error;
mod rlimit;
mod sys;
mod timeout;
mod unshare;

#[cfg(feature = "landlock")]
mod landlock;

#[cfg(feature = "seccomp")]
mod seccomp;

use std::collections::HashMap;
use std::ffi::CString;
use std::io::prelude::*;
use std::io::{PipeReader, PipeWriter};
use std::process;
use std::time::Instant;

use crate::runc::error::*;
use crate::runc::sys::{ForkResult, Pid, PtraceEvent, Signal, UsageWho, WaitStatus};
use crate::{Command, Container, ExitStatus, ProcPidSmapsRollup, ProcPidStatus, Runctl, Rusage};

macro_rules! process_exit {
    ($err:ident) => {{
        let err = format!("hakoniwa: {}\n", $err);
        _ = sys::write_stderr(err.as_bytes());
        process::exit(ExitStatus::FAILURE)
    }};
}

const PTRACE_EVENT_EXIT: i32 = PtraceEvent::PTRACE_EVENT_EXIT as i32;

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
        Err(err) => ExitStatus::new_failure(&err.to_string()),
    };

    let config = bincode::config::standard();
    let encoded: Vec<u8> = match bincode::serde::encode_to_vec(&status, config) {
        Ok(val) => val,
        Err(err) => process_exit!(err),
    };

    // Assume that the encoded message will not exceed the capacity of the pipe
    // buffer (usually 65,536 bytes), so the writer will not be blocked.
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
        sys::dup2_stdin(&stdin)?;
        drop(stdin);
    }
    if let Some(stdout) = stdout.take() {
        sys::dup2_stdout(&stdout)?;
        drop(stdout);
    }
    if let Some(stderr) = stderr.take() {
        sys::dup2_stderr(&stderr)?;
        drop(stderr);
    }

    // Die with parent.
    sys::set_pdeathsig(Signal::SIGKILL)?;

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
    match sys::fork()? {
        ForkResult::Parent { child, .. } => reap(child, command, container),
        ForkResult::Child => match spawn(command, container) {
            Ok(_) => unreachable!(),
            Err(err) => process_exit!(err),
        },
    }
}

fn reap(child: Pid, command: &Command, container: &Container) -> Result<ExitStatus> {
    // Set PTRACE_O_TRACEEXIT option for the internal process.
    if container.needs_childp_traceexit() {
        let ws = sys::waitpid(child)?;
        match ws {
            WaitStatus::Exited(..) => return Ok(ExitStatus::from_wait_status(&ws, command)),
            WaitStatus::Signaled(..) => return Ok(ExitStatus::from_wait_status(&ws, command)),
            WaitStatus::Stopped(pid, Signal::SIGSTOP) if pid == child => {
                sys::ptrace_traceexit(pid)?;
                sys::ptrace_cont(pid, None)?;
            }
            _ => return Ok(ExitStatus::new_failure(&format!("waitpid(..) => {ws:?}"))),
        }
    }

    // Set a time limit for the internal process.
    if let Some(timeout) = command.wait_timeout {
        timeout::timeout(child, timeout)?;
    }

    // Wait for the internal process to finish.
    let mut proc_pid_smaps_rollup = None;
    let mut proc_pid_status = None;
    let started_at = Instant::now();
    let status = loop {
        let ws = sys::waitpid(child)?;
        match ws {
            WaitStatus::Exited(..) => break ExitStatus::from_wait_status(&ws, command),
            WaitStatus::Signaled(..) => break ExitStatus::from_wait_status(&ws, command),
            WaitStatus::PtraceEvent(pid, Signal::SIGTRAP, PTRACE_EVENT_EXIT) if pid == child => {
                proc_pid_smaps_rollup = reap_proc_smaps_rollup(pid, container)?;
                proc_pid_status = reap_proc_status(pid, container)?;
                sys::ptrace_cont(pid, None)?
            }
            WaitStatus::Stopped(pid, Signal::SIGTRAP) => sys::ptrace_cont(pid, None)?,
            WaitStatus::Stopped(pid, signal) => sys::ptrace_cont(pid, Some(signal))?,
            _ => break ExitStatus::new_failure(&format!("waitpid(..) => {ws:?}")),
        };
    };

    // Get resource usage.
    let real_time = started_at.elapsed();
    let rusage = sys::getrusage(UsageWho::RUSAGE_CHILDREN)?;

    // Build the exit status of the internal process.
    Ok(ExitStatus {
        code: status.code,
        reason: status.reason,
        exit_code: status.exit_code,
        rusage: Rusage::from_nix_rusage(rusage, real_time),
        proc_pid_smaps_rollup,
        proc_pid_status,
    })
}

fn reap_proc_smaps_rollup(pid: Pid, container: &Container) -> Result<Option<ProcPidSmapsRollup>> {
    if !container.runctl.contains(&Runctl::GetProcPidSmapsRollup) {
        return Ok(None);
    }

    let mount = container.get_mount_newproc();
    let root = if let Some(mount) = mount {
        format!("{}/1", mount.target)
    } else {
        format!("/proc/{pid}")
    };

    let process = procfs::process::Process::new_with_root(root.into())?;
    let smaps = process.smaps_rollup()?;
    Ok(ProcPidSmapsRollup::from_procfs_smaps_rollup(smaps))
}

fn reap_proc_status(pid: Pid, container: &Container) -> Result<Option<ProcPidStatus>> {
    if !container.runctl.contains(&Runctl::GetProcPidStatus) {
        return Ok(None);
    }

    let mount = container.get_mount_newproc();
    let root = if let Some(mount) = mount {
        format!("{}/1", mount.target)
    } else {
        format!("/proc/{pid}")
    };

    let process = procfs::process::Process::new_with_root(root.into())?;
    let status = process.status()?;
    Ok(ProcPidStatus::from_procfs_status(status))
}

fn spawn(command: &Command, container: &Container) -> Result<()> {
    // Die with parent.
    sys::set_pdeathsig(Signal::SIGKILL)?;

    // Mount procfs, etc.
    unshare::tidyup(container)?;

    // Switch to the working directory.
    if let Some(dir) = command.get_current_dir() {
        sys::chdir(dir)?
    };

    // Turn this process into a tracee.
    if container.needs_childp_traceexit() {
        sys::traceme()?;
        sys::sigraise(Signal::SIGSTOP)?;
    }

    // Reset SIGPIPE to SIG_DFL.
    sys::reset_sigpipe()?;

    // Set resource limit.
    rlimit::setrlimit(container)?;

    // Restrict ambient rights (e.g. global filesystem access).
    #[cfg(feature = "landlock")]
    landlock::load(container)?;

    // Restrict syscalls.
    #[cfg(feature = "seccomp")]
    seccomp::load(container)?;

    // Set the no_new_privs bit.
    #[cfg(not(feature = "seccomp"))]
    if !container.runctl.contains(&Runctl::AllowNewPrivs) {
        sys::set_no_new_privs()?
    }

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
        let env = CString::new(format!("{k}={v}"))?;
        envp.push(env);
    }

    sys::execve(&prog, &argv, &envp)
}
