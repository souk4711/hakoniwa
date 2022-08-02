use nix::{sys::resource, sys::resource::Resource};

use crate::{tryfn, Limits, Result};

pub fn init(limits: &Limits) -> Result<()> {
    setrlimit(Resource::RLIMIT_AS, limits.r#as)?;
    setrlimit(Resource::RLIMIT_CPU, limits.cpu)?;
    setrlimit(Resource::RLIMIT_CORE, limits.core)?;
    setrlimit(Resource::RLIMIT_FSIZE, limits.fsize)?;
    setrlimit(Resource::RLIMIT_NOFILE, limits.nofile)
}

fn setrlimit(resource: Resource, limit: Option<u64>) -> Result<()> {
    match limit {
        Some(limit) => {
            tryfn!(
                resource::setrlimit(resource, limit, limit),
                "setrlimit({:?}, {}, {})",
                resource,
                limit,
                limit
            )
        }
        None => Ok(()),
    }
}
