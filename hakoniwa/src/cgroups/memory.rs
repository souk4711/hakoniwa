use super::error::*;

/// Represents the cgroup subsystem memory.
#[derive(Clone, Default, Debug)]
pub struct Memory {
    reservation: Option<i64>,
    limit: Option<i64>,
    swap: Option<i64>,
}

impl Memory {
    /// Sets soft limit of memory usage in bytes, a.k.a reservation.
    ///
    /// memory.soft_limit_in_bytes (v1) = memory.low (v2)
    ///                                 = MemoryLow (systemd)
    pub fn reservation(&mut self, val: i64) -> &mut Self {
        self.reservation = Some(val);
        self
    }

    /// Sets hard limit of memory usage in bytes, a.k.a physical ram usage.
    ///
    /// memory.limit_in_bytes (v1) = memory.max (v2)
    ///                            = MemoryMax (systemd)
    pub fn limit(&mut self, val: i64) -> &mut Self {
        self.limit = Some(val);
        self
    }

    /// Sets limit of memory+swap usage in bytes.
    ///
    /// memory.memsw.limit_in_bytes (v1) = memory.max + memory.swap.max (v2)
    ///                                  = MemoryMax + MemorySwapMax (systemd)
    pub fn swap(&mut self, val: i64) -> &mut Self {
        self.swap = Some(val);
        self
    }

    /// Build.
    pub(crate) fn build(&self) -> Result<oci_spec::runtime::LinuxMemory> {
        let mut builder = oci_spec::runtime::LinuxMemoryBuilder::default();
        if let Some(val) = self.reservation {
            builder = builder.reservation(val);
        }
        if let Some(val) = self.limit {
            builder = builder.limit(val);
        }
        if let Some(val) = self.swap {
            builder = builder.swap(val);
        }
        Ok(builder.build()?)
    }
}
