use nix::sys::resource::Resource;

use crate::{Limits, Result};

pub fn init(limits: &Limits) -> Result<()> {
    super::syscall::setrlimit(Resource::RLIMIT_AS, limits.r#as)?;
    super::syscall::setrlimit(Resource::RLIMIT_CPU, limits.cpu)?;
    super::syscall::setrlimit(Resource::RLIMIT_CORE, limits.core)?;
    super::syscall::setrlimit(Resource::RLIMIT_FSIZE, limits.fsize)?;
    super::syscall::setrlimit(Resource::RLIMIT_NOFILE, limits.nofile)
}
