use nix::sys::resource;
use serde::Deserialize;

#[derive(Deserialize, Clone, Default)]
pub struct Limits {
    pub(crate) r#as: Option<resource::rlim_t>,   // RLIMIT_AS
    pub(crate) core: Option<resource::rlim_t>,   // RLIMIT_CORE
    pub(crate) cpu: Option<resource::rlim_t>,    // RLIMIT_CPU
    pub(crate) fsize: Option<resource::rlim_t>,  // RLIMIT_FSIZE
    pub(crate) nofile: Option<resource::rlim_t>, // RLIMIT_NOFILE
}
