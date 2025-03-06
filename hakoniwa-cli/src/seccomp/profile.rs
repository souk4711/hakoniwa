use serde::Deserialize;

#[derive(Deserialize)]
pub(crate) struct Profile {
    #[serde(rename = "defaultAction")]
    pub(crate) default_action: String,
    #[serde(rename = "defaultErrnoRet", default)]
    pub(crate) default_errno_ret: i32,
    #[serde(rename = "archMap", default)]
    pub(crate) arch_map: Vec<Architecture>,
    #[serde(rename = "syscalls", default)]
    pub(crate) syscalls: Vec<Syscall>,
}

#[derive(Deserialize)]
pub(crate) struct Architecture {
    #[serde(rename = "architecture")]
    pub(crate) arch: String,
    #[serde(rename = "subArchitectures", default)]
    pub(crate) sub_arches: Vec<String>,
}

#[derive(Deserialize)]
pub(crate) struct Syscall {
    #[serde(rename = "names")]
    pub(crate) names: Vec<String>,
    #[serde(rename = "action")]
    pub(crate) action: String,
    #[serde(rename = "errnoRet", default)]
    pub(crate) errno_ret: i32,
    #[serde(rename = "args", default)]
    pub(crate) args: Vec<SyscallArg>,
    #[serde(rename = "includes", default)]
    pub(crate) includes: Filter,
    #[serde(rename = "excludes", default)]
    pub(crate) excludes: Filter,
}

#[derive(Deserialize)]
pub(crate) struct SyscallArg {
    #[serde(rename = "index")]
    pub(crate) index: u32,
    #[serde(rename = "value")]
    pub(crate) value: u64,
    #[serde(rename = "valueTwo", default)]
    pub(crate) value_two: u64,
    #[serde(rename = "op")]
    pub(crate) op: String,
}

#[derive(Deserialize, Default)]
pub(crate) struct Filter {
    #[serde(rename = "arches", default)]
    pub(crate) arches: Vec<String>,
    #[serde(rename = "caps", default)]
    pub(crate) caps: Vec<String>,
}
