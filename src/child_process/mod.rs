mod exec;
mod namespaces;
mod rlimits;
mod syscall;

use crate::{Executor, Result};

pub fn run(executor: &Executor) -> Result<()> {
    namespaces::init(
        &executor.namespaces,
        &executor.uid_mappings,
        &executor.gid_mappings,
        &executor.rootfs,
        &executor.mounts,
        &executor.dir,
    )?;
    rlimits::init(&executor.limits)?;
    exec::exec(&executor.prog, &executor.argv)
}
