use nix::sys::resource::Resource;

use crate::{
    child_process::{error::Result, syscall},
    Limits,
};

pub(crate) fn init(limits: &Limits) -> Result<()> {
    syscall::setrlimit(Resource::RLIMIT_AS, limits.r#as)?;
    syscall::setrlimit(Resource::RLIMIT_CPU, limits.cpu)?;
    syscall::setrlimit(Resource::RLIMIT_CORE, limits.core)?;
    syscall::setrlimit(Resource::RLIMIT_FSIZE, limits.fsize)?;
    syscall::setrlimit(Resource::RLIMIT_NOFILE, limits.nofile)
}
