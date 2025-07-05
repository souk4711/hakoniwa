use serde::{Deserialize, Serialize};

/// Accumulated smaps stats for all mappings.
///
/// [proc]: https://docs.kernel.org/filesystems/proc.html
#[derive(Serialize, Deserialize, Clone, Default, Debug)]
pub struct SmapsRollup {
    /// The amount of the mapping that is currently resident in RAM, in kilobytes.
    pub rss: u64,

    /// The number of clean shared pages in Rss.
    pub shared_clean: u64,

    /// The number of dirty shared pages in Rss.
    pub shared_dirty: u64,

    /// The number of clean private pages in Rss.
    pub private_clean: u64,

    /// The number of dirty private pages in Rss.
    pub private_dirty: u64,

    /// The process's proportional share of this mapping, in kilobytes.
    pub pss: u64,

    /// The portion of Pss which consists of dirty pages.
    pub pss_dirty: u64,

    /// The sum of the Pss field of anon type.
    pub pss_anon: u64,

    /// The sum of the Pss field of file type.
    pub pss_file: u64,

    /// The sum of the Pss field of shmem type.
    pub pss_shmem: u64,
}

impl SmapsRollup {
    pub(crate) fn from_procfs_smaps_rollup(
        smaps_rollup: procfs::process::SmapsRollup,
    ) -> Option<Self> {
        let memory_maps = smaps_rollup.memory_map_rollup.0;
        if memory_maps.is_empty() {
            return None;
        }

        let mut r = Self::default();
        for (k, v) in &memory_maps[0].extension.map {
            let v = v / 1024;
            match k.as_str() {
                "Rss" => r.rss = v,
                "Shared_Dirty" => r.shared_dirty = v,
                "Shared_Clean" => r.shared_clean = v,
                "Private_Dirty" => r.private_dirty = v,
                "Private_Clean" => r.private_clean = v,
                "Pss" => r.pss = v,
                "Pss_Dirty" => r.pss_dirty = v,
                "Pss_Anon" => r.pss_anon = v,
                "Pss_File" => r.pss_file = v,
                "Pss_Shmem" => r.pss_shmem = v,
                _ => {}
            }
        }
        Some(r)
    }
}
