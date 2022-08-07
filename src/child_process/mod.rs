mod error;
mod exec;
mod namespaces;
mod rlimits;
mod syscall;

use nix::unistd::{self, ForkResult, Pid};
use std::process;

use crate::Executor;

pub fn run(executor: &Executor) -> error::Result<()> {
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
    match syscall::fork() {
        Ok(ForkResult::Parent { child, .. }) => run_in_child(child),
        Ok(ForkResult::Child) => match run_in_grandchild(executor) {
            Ok(_) => unreachable!(),
            Err(err) => {
                let err = format!("hakoniwa: {}\n", err);
                unistd::write(libc::STDERR_FILENO, err.as_bytes()).ok();
                process::exit(Executor::EXITCODE_FAILURE)
            }
        },
        Err(err) => Err(err),
    }
}

fn run_in_child(grandchild: Pid) -> error::Result<()> {
    syscall::waitpid(grandchild)?;
    process::exit(0)
}

fn run_in_grandchild(executor: &Executor) -> error::Result<()> {
    namespaces::reinit(
        &executor.namespaces,
        &executor.uid_mappings,
        &executor.gid_mappings,
        &executor.mounts,
    )?;
    rlimits::init(&executor.limits)?;
    exec::exec(&executor.prog, &executor.argv, &executor.envp)
}
