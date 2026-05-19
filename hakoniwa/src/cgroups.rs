//! Configure cgroups profile.
//!
//! The current api is based on the [OCI Runtime Spec] which uses CGroups
//! v1. However, the underlying technology is CGroups v2 + Systemd. The
//! [conversions][crun] happen when any limits are applied.
//!
//! [OCI Runtime Spec]: https://github.com/opencontainers/runtime-spec/blob/v1.3.0/config-linux.md#control-groups
//! [crun]: https://github.com/containers/crun/blob/1.27/crun.1.md#memory-controller

mod conv;
mod cpu;
mod error;
mod manager;
mod memory;
mod pids;
mod resources;

pub(crate) use manager::Manager;

pub use cpu::Cpu;
pub use error::Error;
pub use memory::Memory;
pub use pids::Pids;
pub use resources::Resources;

use crate::{Container, error::*};
use nix::unistd::Pid;

pub(crate) fn mainp_setup_cgroups(container: &Container, child: Pid) -> Result<Manager> {
    let resources = container
        .cgroups_resources
        .clone()
        .expect("Container#cgroups_resources is some");

    let cgroup = Manager::new(&format!("{child}")).map_err(ProcessErrorKind::SetupCgroupsFailed)?;
    cgroup
        .apply(child, &resources)
        .map_err(ProcessErrorKind::SetupCgroupsFailed)?;

    Ok(cgroup)
}
