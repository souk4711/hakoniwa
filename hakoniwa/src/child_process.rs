mod error;
mod exec;
mod namespaces;
mod rlimits;
mod seccomp;
mod syscall;
mod timeout;

use chrono::prelude::*;
use nix::{
    sys::resource::UsageWho,
    sys::signal::Signal,
    sys::wait::WaitStatus,
    unistd::{ForkResult, Pid},
};
use std::{os::unix::io::RawFd, process, time::Instant};

use crate::{contrib, Executor, ExecutorResultStatus};

pub(crate) mod result;

pub(crate) fn run(
    executor: &Executor,
    (cpr_reader, cpr_writer): (RawFd, RawFd),
    (out_reader, out_writer): (RawFd, RawFd),
    (err_reader, err_writer): (RawFd, RawFd),
    (in_reader, in_writer): (RawFd, RawFd),
) {
    // Run.
    let cpr = match _run(
        executor,
        (cpr_reader, cpr_writer),
        (out_reader, out_writer),
        (err_reader, err_writer),
        (in_reader, in_writer),
    ) {
        Ok(val) => val,
        Err(err) => result::ChildProcessResult::failure(&err.to_string()),
    };

    // Send all data to parent.
    if let Err(err) = result::ChildProcessResult::send_to(cpr_writer, cpr) {
        let err = format!("hakoniwa: {}\n", err);
        _ = syscall::write(libc::STDERR_FILENO, err.as_bytes());
        _ = syscall::close(cpr_writer);
        process::exit(Executor::EXITCODE_FAILURE)
    }

    // Exit.
    _ = syscall::close(cpr_writer);
    process::exit(0)
}

fn _run(
    executor: &Executor,
    (cpr_reader, cpr_writer): (RawFd, RawFd),
    (out_reader, out_writer): (RawFd, RawFd),
    (err_reader, err_writer): (RawFd, RawFd),
    (in_reader, in_writer): (RawFd, RawFd),
) -> error::Result<result::ChildProcessResult> {
    // Die with parent.
    syscall::prctl_set_pdeathsig(libc::SIGKILL)?;

    // Close unused pipe.
    syscall::close(cpr_reader)?;
    syscall::close(out_reader)?;
    syscall::close(err_reader)?;
    syscall::close(in_writer)?;

    // Redirect stdout.
    syscall::dup2(out_writer, libc::STDOUT_FILENO)?;
    syscall::close(out_writer)?;

    // Redirect stderr.
    syscall::dup2(err_writer, libc::STDERR_FILENO)?;
    syscall::close(err_writer)?;

    // Redirect stdin.
    syscall::dup2(in_reader, libc::STDIN_FILENO)?;
    syscall::close(in_reader)?;

    // Create new session.
    syscall::setsid()?;

    // Create new namespace.
    namespaces::init(
        &executor.container_root_dir,
        &executor.namespaces,
        &executor.hostname,
        &executor.mounts,
    )?;

    // -f, --fork
    //     Fork the specified program as a child process of unshare
    //     rather than running it directly. This is useful when creating
    //     a new PID namespace.
    //
    // [unshare]: https://man7.org/linux/man-pages/man1/unshare.1.html
    match syscall::fork()? {
        ForkResult::Parent { child, .. } => _run_in_child(executor, child),
        ForkResult::Child => match _run_in_grandchild(executor, cpr_writer) {
            Ok(_) => unreachable!(),
            Err(err) => {
                let err = format!("hakoniwa: {}\n", err);
                _ = syscall::write(libc::STDERR_FILENO, err.as_bytes());
                process::exit(Executor::EXITCODE_FAILURE)
            }
        },
    }
}

fn _run_in_child(
    executor: &Executor,
    grandchild: Pid,
) -> error::Result<result::ChildProcessResult> {
    if let Some(timeout) = executor.limits.walltime {
        timeout::init(timeout, grandchild)?;
    }

    let mut r = result::ChildProcessResult::new();
    r.start_time = Some(Utc::now());

    let start_time_instant = Instant::now();
    match syscall::waitpid(grandchild)? {
        WaitStatus::Exited(_, exit_status) => {
            r.status = ExecutorResultStatus::Ok;
            r.reason = String::new();
            r.exit_code = Some(exit_status);
        }
        WaitStatus::Signaled(_, signal, _) => {
            r.status = match signal {
                Signal::SIGKILL => ExecutorResultStatus::TimeLimitExceeded,
                Signal::SIGXCPU => ExecutorResultStatus::TimeLimitExceeded,
                Signal::SIGXFSZ => ExecutorResultStatus::OutputLimitExceeded,
                Signal::SIGSYS => ExecutorResultStatus::RestrictedFunction,
                _ => ExecutorResultStatus::Signaled,
            };
            r.reason = format!("signaled: {}", signal);
            r.exit_code = Some(128 + (signal as i32));
        }
        _ => {
            r.status = ExecutorResultStatus::SandboxSetupError;
            r.reason = String::from("unexpected wait status");
            r.exit_code = Some(Executor::EXITCODE_FAILURE);
        }
    }

    let real_time = start_time_instant.elapsed();
    let rusage = syscall::getrusage(UsageWho::RUSAGE_CHILDREN)?;
    let user_time = rusage.user_time();
    let system_time = rusage.system_time();
    let max_rss = rusage.max_rss();

    r.real_time = Some(real_time);
    r.user_time = Some(contrib::nix::time::from_timeval_into_duration(user_time));
    r.system_time = Some(contrib::nix::time::from_timeval_into_duration(system_time));
    r.max_rss = Some(max_rss);
    Ok(r)
}

fn _run_in_grandchild(executor: &Executor, cpr_writer: RawFd) -> error::Result<()> {
    // Die with parent.
    syscall::prctl_set_pdeathsig(libc::SIGKILL)?;

    // Close unused pipe.
    syscall::close(cpr_writer)?;

    // Exec.
    namespaces::reinit(
        &executor.namespaces,
        &executor.uid_mappings,
        &executor.gid_mappings,
        &executor.mounts,
        &executor.files,
        &executor.dir,
    )?;
    rlimits::init(&executor.limits)?;
    seccomp::init(&executor.seccomp)?;
    exec::exec(&executor.prog, &executor.argv, &executor.envp)
}
