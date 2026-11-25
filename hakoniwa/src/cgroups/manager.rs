use libcgroups::common::{
    AnyCgroupManager, CgroupConfig, CgroupManager, ControllerOpt as CgroupControllerOpt,
    create_cgroup_manager,
};
use nix::unistd::Pid;
use std::path::PathBuf;

use super::{Resources, error::*};

pub(crate) struct Manager {
    manager: AnyCgroupManager,
}

impl Manager {
    /// Creates a cgroup for the container `id`, selecting the backend from the
    /// enabled features and the host:
    ///
    /// - `cgroups-systemd`: a transient scope `hakoniwa.slice:hakoniwa:<id>`,
    ///   when the host is booted with systemd and uses the unified hierarchy.
    /// - `cgroups-v2`: otherwise, `/hakoniwa/<id>` on the unified hierarchy.
    /// - `cgroups-v1`: `/hakoniwa/<id>` on a legacy or hybrid hierarchy.
    ///
    /// A backend that is not compiled in is reported by libcgroups as a
    /// `NotEnabled` error.
    pub(crate) fn new(id: &str) -> Result<Self> {
        let systemd_cgroup = Self::use_systemd();
        let cgroup_path = if systemd_cgroup {
            PathBuf::from(format!("hakoniwa.slice:hakoniwa:{id}"))
        } else {
            PathBuf::from(format!("/hakoniwa/{id}"))
        };

        let manager = create_cgroup_manager(CgroupConfig {
            cgroup_path,
            systemd_cgroup,
            container_name: id.to_string(),
        })?;
        Ok(Self { manager })
    }

    #[cfg(feature = "cgroups-systemd")]
    fn use_systemd() -> bool {
        use libcgroups::common::{CgroupSetup, get_cgroup_setup};
        libcgroups::systemd::booted() && matches!(get_cgroup_setup(), Ok(CgroupSetup::Unified))
    }

    #[cfg(not(feature = "cgroups-systemd"))]
    fn use_systemd() -> bool {
        false
    }

    pub(crate) fn apply(&self, task: Pid, resources: &Resources) -> Result<()> {
        let opts = CgroupControllerOpt {
            resources: &resources.build()?,
            disable_oom_killer: false,
            oom_score_adj: None,
            freezer_state: None,
        };
        self.manager.add_task(task)?;
        self.manager.apply(&opts)?;
        Ok(())
    }
}

impl Drop for Manager {
    fn drop(&mut self) {
        _ = self.manager.remove();
    }
}
