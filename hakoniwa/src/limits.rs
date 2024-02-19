use serde::Deserialize;

#[derive(Deserialize, Clone, Default, Debug)]
#[serde(deny_unknown_fields)]
pub(crate) struct Limits {
    pub(crate) r#as: Option<u64>,     // RLIMIT_AS
    pub(crate) core: Option<u64>,     // RLIMIT_CORE
    pub(crate) cpu: Option<u64>,      // RLIMIT_CPU
    pub(crate) fsize: Option<u64>,    // RLIMIT_FSIZE
    pub(crate) nofile: Option<u64>,   // RLIMIT_NOFILE
    pub(crate) walltime: Option<u64>, // walltime
}
