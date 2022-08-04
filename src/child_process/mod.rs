mod exec;
mod namespaces;
mod rlimits;
mod syscall;

use nix::{sys::wait, unistd, unistd::ForkResult, unistd::Pid};
use std::process;

use crate::{Error, Executor, Result};

pub fn run(executor: &Executor) -> Result<()> {
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
    match { unsafe { unistd::fork() } } {
        Ok(ForkResult::Parent { child, .. }) => run_in_child(child),
        Ok(ForkResult::Child) => match run_in_grandchild(executor) {
            Ok(_) => unreachable!(),
            Err(err) => {
                let err = format!("hakoniwa: {}\n", err);
                unistd::write(libc::STDERR_FILENO, err.as_bytes()).ok();
                process::exit(Executor::EXITCODE_FAILURE)
            }
        },
        Err(err) => {
            let err = format!("fork() => {}", err);
            Err(Error::FnError(err))
        }
    }
}

fn run_in_child(grandchild: Pid) -> Result<()> {
    if let Err(err) = wait::waitpid(grandchild, None) {
        let err = format!("waitpid({}) => {}", grandchild, err);
        return Err(Error::FnError(err));
    }

    process::exit(0)
}

fn run_in_grandchild(executor: &Executor) -> Result<()> {
    namespaces::reinit(
        &executor.namespaces,
        &executor.uid_mappings,
        &executor.gid_mappings,
        &executor.mounts,
    )?;
    rlimits::init(&executor.limits)?;
    exec::exec(&executor.prog, &executor.argv)
}
