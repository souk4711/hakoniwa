use serde::Deserialize;

#[derive(Deserialize)]
pub(crate) struct Profile {
    #[serde(rename = "defaultAction")]
    pub(crate) default_action: String,
    #[serde(rename = "defaultErrnoRet")]
    pub(crate) default_errno_ret: i32,
    #[serde(rename = "archMap")]
    pub(crate) arch_map: Vec<Architecture>,
    #[serde(rename = "syscalls")]
    pub(crate) syscalls: Vec<Syscall>,
}

#[derive(Deserialize)]
pub(crate) struct Architecture {
    #[serde(rename = "architecture")]
    pub(crate) arch: String,
    #[serde(rename = "subArchitectures")]
    pub(crate) sub_arches: Vec<String>,
}

#[derive(Deserialize)]
pub(crate) struct Syscall {
    #[serde(rename = "names")]
    pub(crate) names: Vec<String>,
    #[serde(rename = "action")]
    pub(crate) action: String,
    #[serde(rename = "errnoRet")]
    pub(crate) errno_ret: Option<i32>,
    #[serde(rename = "args")]
    pub(crate) args: Option<Vec<SyscallArg>>,
    #[serde(rename = "includes")]
    pub(crate) includes: Filter,
    #[serde(rename = "excludes")]
    pub(crate) excludes: Filter,
}

#[derive(Deserialize)]
pub(crate) struct SyscallArg {
    #[serde(rename = "index")]
    pub(crate) index: u32,
    #[serde(rename = "value")]
    pub(crate) value: u64,
    #[serde(rename = "valueTwo")]
    pub(crate) value_two: u64,
    #[serde(rename = "op")]
    pub(crate) op: String,
}

#[derive(Deserialize)]
pub(crate) struct Filter {
    #[serde(rename = "arches")]
    pub(crate) arches: Option<Vec<String>>,
    #[serde(rename = "caps")]
    pub(crate) caps: Option<Vec<String>>,
}
