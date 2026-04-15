mod error;
mod notify;
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
use std::os::fd::AsRawFd;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::time::Instant;

use crate::runc::error::*;
use crate::runc::sys::{ForkResult, Pid, PtraceEvent, Signal, UsageWho, WaitStatus};
use crate::stdio::{EndReader, EndWriter};
use crate::{Command, Container, ExitStatus, ProcPidSmapsRollup, ProcPidStatus, Runctl, Rusage};

macro_rules! process_exit_with_status {
    ($status:expr) => {{ unsafe { libc::_exit($status) } }};
}

macro_rules! process_exit_with_failure {
    () => {{ process_exit_with_status!(ExitStatus::FAILURE) }};

    ($err:expr) => {{
        let err = format!("hakoniwa: {}\n", $err);
        _ = sys::write_stderr(err.as_bytes());
        process_exit_with_status!(ExitStatus::FAILURE)
    }};
}

const PTRACE_EVENT_EXIT: i32 = PtraceEvent::PTRACE_EVENT_EXIT as i32;

pub(crate) const FIN: u8 = 0;
pub(crate) const SETUP_UGIDMAP: u8 = 1;
pub(crate) const SETUP_NETWORK: u8 = 1 << 1;
pub(crate) const SETUP_CGROUPS: u8 = 1 << 2;
pub(crate) const SETUP_SUCCESS: u8 = 1 << 7;

pub(crate) fn exec(
    command: &Command,
    container: &Container,
    mut stdin: Option<EndReader>,
    mut stdout: Option<EndWriter>,
    mut stderr: Option<EndWriter>,
    reader: PipeReader,
    writer: PipeWriter,
) {
    let mut writer_opt = Some(writer);
    let status = match exec_imp(
        command,
        container,
        &mut stdin,
        &mut stdout,
        &mut stderr,
        reader,
        &mut writer_opt,
    ) {
        Ok(val) => val,
        Err(err) => ExitStatus::new_failure(&err.to_string()),
    };

    let encoded: Vec<u8> = match postcard::to_allocvec(&status) {
        Ok(val) => val,
        Err(_) => process_exit_with_failure!(),
    };

    // Assume that the encoded message will not exceed the capacity of the pipe
    // buffer (usually 65,536 bytes), so the writer will not be blocked.
    let mut writer = writer_opt.expect("writer is some");
    match writer.write_all(&[FIN]) {
        Ok(_) => {}
        Err(_) => process_exit_with_failure!(),
    };
    match writer.write_all(&encoded) {
        Ok(_) => {}
        Err(_) => process_exit_with_failure!(),
    };
    drop(writer);

    process_exit_with_status!(status.code)
}

fn exec_imp(
    command: &Command,
    container: &Container,
    stdin: &mut Option<EndReader>,
    stdout: &mut Option<EndWriter>,
    stderr: &mut Option<EndWriter>,
    reader: PipeReader,
    writer: &mut Option<PipeWriter>,
) -> Result<ExitStatus> {
    // Redirect standard I/O stream.
    if let Some(stdin) = stdin.take() {
        sys::dup2_stdin(stdin.as_raw_fd())?;
        drop(stdin);
    }
    if let Some(stdout) = stdout.take() {
        sys::dup2_stdout(stdout.as_raw_fd())?;
        drop(stdout);
    }
    if let Some(stderr) = stderr.take() {
        sys::dup2_stderr(stderr.as_raw_fd())?;
        drop(stderr);
    }

    // Close extra FDs.
    let writer_ref = writer.as_ref().expect("writer is some");
    sys::close_extra_fds_exclude(reader.as_raw_fd(), writer_ref.as_raw_fd())?;

    // Die with parent.
    sys::set_pdeathsig(Signal::SIGKILL)?;

    // Unshare namespaces, setup [ug]idmap.
    unshare::newuser(container)?;

    // Notify the main process to setup [ug]idmap, network, cgroups, etc.
    notify::notify_mainp_setup(container, &reader, writer_ref)?;
    drop(reader);

    // Mount rootfs.
    unshare::newns(command, container)?;

    // Fork the specified program as a child process rather than running it
    // directly. This is useful when creating a new PID namespace.
    match sys::fork()? {
        ForkResult::Parent { child, .. } => {
            notify::notify_mainp_setup_success(writer_ref)?;
            reap(child, command, container)
        }
        ForkResult::Child => match spawn(command, container, writer) {
            Ok(_) => unreachable!("runc::exec_imp"),
            Err(err) => process_exit_with_failure!(err),
        },
    }
}

fn reap(child: Pid, command: &Command, container: &Container) -> Result<ExitStatus> {
    // Close unused FDs.
    sys::close_stdin()?;
    sys::close_stdout()?;
    sys::close_stderr()?;

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

fn spawn(command: &Command, container: &Container, writer: &mut Option<PipeWriter>) -> Result<()> {
    // Close FDs.
    drop(writer.take());

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
    if let Some(closure) = &command.program_closure {
        let args = command.get_args();
        let envs = command.get_envs();
        spawn_imp_program_closure(closure, &args, &envs)
    } else {
        let program = command.get_program();
        let args = command.get_args();
        let envs = command.get_envs();
        spawn_imp_program(program, &args, &envs)
    }
}

fn spawn_imp_program_closure<F, S: AsRef<str>>(
    closure: F,
    _args: &[S],
    envs: &HashMap<String, String>,
) -> Result<()>
where
    F: Fn() -> i32 + Send + Sync,
{
    // Prepare envp.
    sys::clearenv()?;
    for (k, v) in envs {
        sys::setenv(k, v)?;
    }

    // Exec closure.
    let mut status = 0;
    let result = catch_unwind(AssertUnwindSafe(|| {
        status = closure();
    }));

    // Exec closure - Success.
    if result.is_ok() {
        process_exit_with_status!(status)
    }

    // Exec closure - Failure.
    let panic_payload = result.unwrap_err();
    if let Some(err) = panic_payload.downcast_ref::<&str>() {
        process_exit_with_failure!(err)
    } else if let Some(err) = panic_payload.downcast_ref::<String>() {
        process_exit_with_failure!(err)
    } else {
        process_exit_with_failure!("unknown panic payload")
    }
}

fn spawn_imp_program<S: AsRef<str>>(
    program: &str,
    args: &[S],
    envs: &HashMap<String, String>,
) -> Result<()> {
    let prog = CString::new(program)?;

    // Prepare argv.
    let mut argv = vec![prog.clone()];
    for arg in args {
        let arg = CString::new(arg.as_ref())?;
        argv.push(arg);
    }

    // Prepare envp.
    let mut envp = vec![];
    for (k, v) in envs {
        let env = CString::new(format!("{k}={v}"))?;
        envp.push(env);
    }

    // Exec program.
    sys::execve(&prog, &argv, &envp)
}
