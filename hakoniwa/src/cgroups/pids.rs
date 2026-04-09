use super::error::*;

/// Represents the cgroup subsystem pids.
#[derive(Clone, Default, Debug)]
pub struct Pids {
    limit: Option<i64>,
}

impl Pids {
    /// Specifies the maximum number of tasks in the cgroup.
    ///
    /// pids.limit (v1) = pids.limit (v2)
    ///                 = TasksMax (systemd)
    pub fn limit(&mut self, val: i64) -> &mut Self {
        self.limit = Some(val);
        self
    }

    /// Build.
    pub(crate) fn build(&self) -> Result<oci_spec::runtime::LinuxPids> {
        let mut builder = oci_spec::runtime::LinuxPidsBuilder::default();
        if let Some(val) = self.limit {
            builder = builder.limit(val);
        }
        Ok(builder.build()?)
    }
}
