mod error;
mod exec;
mod namespaces;
mod rlimits;
mod syscall;

use chrono::prelude::*;
use nix::{
    sys::resource::UsageWho,
    sys::signal::Signal,
    sys::wait::WaitStatus,
    unistd::{self, ForkResult, Pid},
};
use scopeguard::defer;
use std::{os::unix::io::RawFd, process, time::Instant};

use crate::{contrib, Executor, ExecutorResultStatus};

pub mod result;

pub fn run(executor: &Executor, (cpr_reader, cpr_writer): (RawFd, RawFd)) {
    // Die with parent.
    unsafe { libc::prctl(libc::PR_SET_PDEATHSIG, libc::SIGKILL, 0, 0, 0) };

    // Close unused pipe.
    _ = unistd::close(cpr_reader);
    defer! { _ = unistd::close(cpr_writer); }

    // Run.
    let cpr = match _run(executor, cpr_writer) {
        Ok(val) => val,
        Err(err) => result::ChildProcessResult::failure(&err.to_string()),
    };

    // Send all data to parent.
    if let Err(err) = result::ChildProcessResult::send_to(cpr_writer, cpr) {
        let err = format!("hakoniwa: {}\n", err);
        unistd::write(libc::STDERR_FILENO, err.as_bytes()).ok();
        process::exit(Executor::EXITCODE_FAILURE)
    }

    // Exit.
    process::exit(0)
}

fn _run(executor: &Executor, cpr_writer: RawFd) -> error::Result<result::ChildProcessResult> {
    // Create new namespace.
    namespaces::init(
        &executor.namespaces,
        &executor.uid_mappings,
        &executor.gid_mappings,
        &executor.hostname,
        &executor.rootfs,
        &executor.mounts,
        &executor.dir,
    )?;

    // -f, --fork
    //     Fork the specified program as a child process of unshare
    //     rather than running it directly. This is useful when creating
    //     a new PID namespace.
    //
    // [unshare]: https://man7.org/linux/man-pages/man1/unshare.1.html
    match syscall::fork()? {
        ForkResult::Parent { child, .. } => _run_in_child(child),
        ForkResult::Child => match _run_in_grandchild(executor, cpr_writer) {
            Ok(_) => unreachable!(),
            Err(err) => {
                let err = format!("hakoniwa: {}\n", err);
                unistd::write(libc::STDERR_FILENO, err.as_bytes()).ok();
                process::exit(Executor::EXITCODE_FAILURE)
            }
        },
    }
}

fn _run_in_child(grandchild: Pid) -> error::Result<result::ChildProcessResult> {
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

    let rusage = syscall::getrusage(UsageWho::RUSAGE_CHILDREN)?;
    let max_rss = rusage.max_rss();
    let user_time = rusage.user_time();
    let system_time = rusage.system_time();

    r.max_rss = Some(max_rss);
    r.user_time = Some(contrib::nix::from_timeval_into_duration(user_time));
    r.system_time = Some(contrib::nix::from_timeval_into_duration(system_time));
    r.real_time = Some(start_time_instant.elapsed());
    Ok(r)
}

fn _run_in_grandchild(executor: &Executor, cpr_writer: RawFd) -> error::Result<()> {
    // Die with parent.
    unsafe { libc::prctl(libc::PR_SET_PDEATHSIG, libc::SIGKILL, 0, 0, 0) };

    // Close unused pipe.
    _ = unistd::close(cpr_writer);

    // .
    namespaces::reinit(
        &executor.namespaces,
        &executor.uid_mappings,
        &executor.gid_mappings,
        &executor.mounts,
    )?;
    rlimits::init(&executor.limits)?;
    exec::exec(&executor.prog, &executor.argv, &executor.envp)
}
