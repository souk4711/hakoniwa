use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Information about resource usage.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Rusage {
    /// Wall clock time.
    pub real_time: Duration,

    /// Total amount of time spent executing in user mode.
    pub user_time: Duration,

    /// Total amount of time spent executing in kernel mode.
    pub system_time: Duration,

    /// The resident set size at its peak, in kilobytes.
    pub max_rss: i64,
}

impl Rusage {
    pub(crate) fn from_nix_rusage(
        rusage: nix::sys::resource::Usage,
        real_time: Duration,
    ) -> Option<Self> {
        let user_time = rusage.user_time();
        let user_time = Duration::new(
            user_time.tv_sec() as u64,
            (user_time.tv_usec() * 1000) as u32,
        );

        let system_time = rusage.system_time();
        let system_time = Duration::new(
            system_time.tv_sec() as u64,
            (system_time.tv_usec() * 1000) as u32,
        );

        Some(Self {
            real_time,
            user_time,
            system_time,
            max_rss: rusage.max_rss(),
        })
    }
}
