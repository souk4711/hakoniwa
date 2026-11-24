//! Configure cgroups profile.
//!
//! The current api is based on the [OCI Runtime Spec] which uses CGroups
//! v1. The limits are applied through one of the enabled backends, and the
//! [conversions][crun] to CGroups v2 happen when any limits are applied:
//!
//! - `cgroups-systemd`: a systemd transient scope, when the host is booted
//!   with systemd and uses the unified hierarchy.
//! - `cgroups-v2`: cgroupfs on the unified hierarchy.
//! - `cgroups-v1`: cgroupfs on a legacy or hybrid hierarchy. Limits for
//!   controllers that are not mounted on the host are skipped with a warning.
//!
//! `cgroups-systemd` also builds the cgroupfs v2 backend, which is used on a
//! unified hierarchy when systemd is not running.
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

use crate::{Container, Runctl, error::*};
use nix::unistd::Pid;

pub(crate) fn mainp_setup_cgroups(container: &Container, child: Pid) -> Result<Option<Manager>> {
    let result = mainp_setup_cgroups_imp(container, child);
    if let Err(err) = result {
        if container.runctl.contains(&Runctl::IgnoreCgroupSetupFailed) {
            log::debug!("Ignoring cgroups setup failure: {}", err);
            return Ok(None);
        }
        return Err(err);
    }

    let cgroup = result.expect("result is ok");
    Ok(Some(cgroup))
}

fn mainp_setup_cgroups_imp(container: &Container, child: Pid) -> Result<Manager> {
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
