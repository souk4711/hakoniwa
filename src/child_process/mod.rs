mod exec;
mod mounts;
mod namespace;
mod rlimits;

use nix::Result;

use crate::executor::Executor;

pub fn run(executor: &Executor) -> Result<()> {
    namespace::init(&executor.namespaces)?;
    mounts::init(&executor.dir)?;
    rlimits::init(&executor.limits)?;
    exec::exec(&executor.prog, &executor.argv)
}
