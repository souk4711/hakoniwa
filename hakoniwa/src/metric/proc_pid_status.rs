use serde::{Deserialize, Serialize};

/// Memory usage and status information.
///
/// [proc]: https://docs.kernel.org/filesystems/proc.html
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ProcPidStatus {
    /// Peak virtual memory size by kibibytes.
    pub vmpeak: u64,

    /// Virtual memory size by kibibytes.
    pub vmsize: u64,

    /// Peak resident set size by kibibytes (“high water mark”).
    pub vmhwm: u64,

    /// Resident set size by kibibytes. Note that the value here is the sum of RssAnon, RssFile, and RssShmem.
    pub vmrss: u64,

    /// Size of data by kibibytes.
    pub vmdata: u64,

    /// Size of stack by kibibytes.
    pub vmstk: u64,

    /// Size of text seg‐ments by kibibytes.
    pub vmexe: u64,

    /// Shared library code size by kibibytes.
    pub vmlib: u64,

    /// Page table entries size by kibibytes.
    pub vmpte: u64,

    /// Swapped-out virtual memory size by anonymous private pages by kibibytes; shmem swap usage is not included.
    pub vmswap: u64,

    /// Size of resident anonymous memory by kibibytes.
    pub rssanon: u64,

    /// Size of resident file mappings by kibibytes.
    pub rssfile: u64,

    /// Size of resident shared memory by kibibytes.
    pub rssshmem: u64,
}

impl ProcPidStatus {
    pub(crate) fn from_procfs_status(status: procfs::process::Status) -> Option<Self> {
        Some(Self {
            vmpeak: status.vmpeak.unwrap_or_default(),
            vmsize: status.vmsize.unwrap_or_default(),
            vmhwm: status.vmhwm.unwrap_or_default(),
            vmrss: status.vmrss.unwrap_or_default(),
            vmdata: status.vmdata.unwrap_or_default(),
            vmstk: status.vmstk.unwrap_or_default(),
            vmexe: status.vmexe.unwrap_or_default(),
            vmlib: status.vmlib.unwrap_or_default(),
            vmpte: status.vmpte.unwrap_or_default(),
            vmswap: status.vmswap.unwrap_or_default(),
            rssanon: status.rssanon.unwrap_or_default(),
            rssfile: status.rssfile.unwrap_or_default(),
            rssshmem: status.rssshmem.unwrap_or_default(),
        })
    }
}
