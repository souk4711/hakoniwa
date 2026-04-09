use super::error::*;

/// Represents the cgroup subsystems cpu.
#[derive(Clone, Default, Debug)]
pub struct Cpu {
    shares: Option<u64>,
    period: Option<u64>,
    quota: Option<i64>,
}

impl Cpu {
    /// Specifies a relative value that determines how much CPU time a process gets
    /// in relation to its peers when the CPU is saturated. The default value is
    /// 100, with a range of 1 to 10,000.
    ///
    /// cpu.weight = 10^((log2(cpu.shares)^2 + 125 * log2(cpu.shares)) / 612.0 - 7.0 / 34.0)
    ///
    /// See also [Cpu::shares].
    pub fn weight(&mut self, val: u64) -> &mut Self {
        // crate libcgroups uses cpu.shares as the underlying value. We therefore need to
        // convert cpu.weight to cpu.shares.
        self.shares = Some(super::conv::convert_weight_to_shares(val));
        self
    }

    /// Specifies a relative value that determines how much CPU time a process gets
    /// in relation to its peers when the CPU is saturated. The default value is
    /// 1024, with a range of 2 to 262,144.
    ///
    /// cpu.weight = 10^((log2(cpu.shares)^2 + 125 * log2(cpu.shares)) / 612.0 - 7.0 / 34.0)
    ///
    /// See also [Cpu::weight].
    pub fn shares(&mut self, val: u64) -> &mut Self {
        self.shares = Some(val);
        self
    }

    /// Specifies a period of time in microseconds for how regularly a cgroup's
    /// access to CPU resources should be reallocated (CFS scheduler only). Default
    /// period is usually 100,000 us (100 ms).
    ///
    /// cpu.cfs_period_us (v1) = 2nd value of cpu.max (v2)
    ///                        = CPUQuotaPeriodUSec (systemd)
    pub fn period(&mut self, val: u64) -> &mut Self {
        self.period = Some(val);
        self
    }

    /// Specifies the total amount of time in microseconds for which all tasks
    /// in a cgroup can run during one period (as defined by period).
    ///
    /// cpu.cfs_quota_us (v1) = 1st value of cpu.max (v2)
    ///                       = CPUQuota * cpu.cfs_period_us (systemd)
    pub fn quota(&mut self, val: i64) -> &mut Self {
        self.quota = Some(val);
        self
    }

    /// Build.
    pub(crate) fn build(&self) -> Result<oci_spec::runtime::LinuxCpu> {
        let mut builder = oci_spec::runtime::LinuxCpuBuilder::default();
        if let Some(val) = self.shares {
            builder = builder.shares(val);
        }
        if let Some(val) = self.period {
            builder = builder.period(val);
        }
        if let Some(val) = self.quota {
            builder = builder.quota(val);
        }
        Ok(builder.build()?)
    }
}
