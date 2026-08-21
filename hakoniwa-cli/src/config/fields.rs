use nix::unistd::{Gid, Uid};
use serde::Deserialize;
use std::env;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CfgConfig {
    #[serde(rename = "@include", default)]
    pub(crate) includes: Vec<String>,

    #[serde(rename = "namespaces", default)]
    pub(crate) namespaces: Vec<CfgNamespace>,
    #[serde(rename = "mounts", default)]
    pub(crate) mounts: Vec<CfgMount>,
    #[serde(rename = "filesystem")]
    pub(crate) filesystem: Option<CfgFileSystem>,
    #[serde(rename = "envs", default)]
    pub(crate) envs: Vec<CfgEnv>,
    #[serde(rename = "network")]
    pub(crate) network: Option<CfgNetwork>,
    #[serde(rename = "landlock")]
    pub(crate) landlock: Option<CfgLandlock>,

    #[serde(rename = "rootdir")]
    pub(crate) rootdir: Option<CfgRootDir>,
    #[serde(rename = "uidmaps", default)]
    pub(crate) uidmaps: Vec<CfgUidMap>,
    #[serde(rename = "gidmaps", default)]
    pub(crate) gidmaps: Vec<CfgGidMap>,
    #[serde(rename = "hostname")]
    pub(crate) hostname: Option<String>,
    #[serde(rename = "limits", default)]
    pub(crate) limits: Vec<CfgLimit>,
    #[serde(rename = "seccomp", default)]
    pub(crate) seccomp: CfgSeccomp,
    #[serde(rename = "command", default)]
    pub(crate) command: CfgCommand,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CfgInclude {
    #[serde(rename = "namespaces", default)]
    pub(crate) namespaces: Vec<CfgNamespace>,
    #[serde(rename = "mounts", default)]
    pub(crate) mounts: Vec<CfgMount>,
    #[serde(rename = "filesystem")]
    pub(crate) filesystem: Option<CfgFileSystem>,
    #[serde(rename = "envs", default)]
    pub(crate) envs: Vec<CfgEnv>,
    #[serde(rename = "network")]
    pub(crate) network: Option<CfgNetwork>,
    #[serde(rename = "landlock")]
    pub(crate) landlock: Option<CfgLandlock>,
}

#[derive(Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub(crate) struct CfgNamespace {
    #[serde(rename = "type")]
    pub(crate) nstype: String,
    #[serde(rename = "share", default)]
    pub(crate) share: bool,
}

#[derive(Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub(crate) struct CfgMount {
    #[serde(rename = "source")]
    pub(crate) source: String,
    #[serde(rename = "destination")]
    pub(crate) destination: Option<String>,
    #[serde(rename = "type", default)]
    pub(crate) fstype: String,
    #[serde(rename = "rw", default)]
    pub(crate) rw: bool,
}

#[derive(Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub(crate) struct CfgFileSystem {
    #[serde(rename = "files", default)]
    pub(crate) files: Vec<CfgFileSystemFile>,
    #[serde(rename = "dirs", default)]
    pub(crate) dirs: Vec<CfgFileSystemDir>,
    #[serde(rename = "symlinks", default)]
    pub(crate) symlinks: Vec<CfgFileSystemSymlink>,
}

#[derive(Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub(crate) struct CfgFileSystemFile {
    #[serde(rename = "destination")]
    pub(crate) destination: String,
    #[serde(rename = "contents")]
    pub(crate) contents: String,
}

#[derive(Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub(crate) struct CfgFileSystemDir {
    #[serde(rename = "destination")]
    pub(crate) destination: String,
}

#[derive(Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub(crate) struct CfgFileSystemSymlink {
    #[serde(rename = "original")]
    pub(crate) original: String,
    #[serde(rename = "link")]
    pub(crate) link: String,
}

#[derive(Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub(crate) struct CfgEnv {
    #[serde(rename = "name")]
    pub(crate) name: String,
    #[serde(rename = "value")]
    pub(crate) value: Option<String>,
}

#[derive(Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub(crate) struct CfgNetwork {
    #[serde(rename = "mode")]
    pub(crate) mode: String,
    #[serde(rename = "options", default)]
    pub(crate) options: Vec<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CfgLandlock {
    #[serde(rename = "resources", default)]
    pub(crate) resources: Vec<CfgLandlockResource>,
    #[serde(rename = "fs", default)]
    pub(crate) fs: Vec<CfgLandlockFsRule>,
    #[serde(rename = "net", default)]
    pub(crate) net: Vec<CfgLandlockNetRule>,
}

#[derive(Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub(crate) struct CfgLandlockResource {
    #[serde(rename = "type")]
    pub(crate) rtype: String,
    #[serde(rename = "unrestrict", default)]
    pub(crate) unrestrict: bool,
}

#[derive(Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub(crate) struct CfgLandlockFsRule {
    #[serde(rename = "path")]
    pub(crate) path: String,
    #[serde(rename = "access")]
    pub(crate) access: String,
}

#[derive(Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub(crate) struct CfgLandlockNetRule {
    #[serde(rename = "port")]
    pub(crate) port: u16,
    #[serde(rename = "access")]
    pub(crate) access: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CfgRootDir {
    #[serde(rename = "path")]
    pub(crate) path: String,
    #[serde(rename = "rw", default)]
    pub(crate) rw: bool,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CfgUidMap {
    #[serde(rename = "container_id")]
    pub(crate) container_id: u32,
    #[serde(rename = "host_id")]
    pub(crate) host_id: Option<u32>,
    #[serde(rename = "count")]
    pub(crate) count: Option<u32>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CfgGidMap {
    #[serde(rename = "container_id")]
    pub(crate) container_id: u32,
    #[serde(rename = "host_id")]
    pub(crate) host_id: Option<u32>,
    #[serde(rename = "count")]
    pub(crate) count: Option<u32>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CfgLimit {
    #[serde(rename = "type")]
    pub(crate) rtype: String,
    #[serde(rename = "value")]
    pub(crate) value: u64,
}

#[derive(Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub(crate) struct CfgSeccomp {
    #[serde(rename = "path")]
    pub(crate) path: Option<String>,
}

#[derive(Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub(crate) struct CfgCommand {
    #[serde(rename = "cmdline")]
    pub(crate) cmdline: Vec<String>,
    #[serde(rename = "cwd")]
    pub(crate) cwd: Option<String>,
}

impl CfgEnv {
    pub(crate) fn unwrap_or_default(self) -> (String, String) {
        match self.value {
            Some(v) => (self.name, v),
            None => match env::var(&self.name) {
                Ok(v) => (self.name, v),
                Err(_) => (self.name, "".to_string()),
            },
        }
    }
}

impl CfgUidMap {
    pub(crate) fn unwrap_or_default(self) -> (u32, u32, u32) {
        (
            self.container_id,
            self.host_id.unwrap_or(Uid::current().as_raw()),
            self.count.unwrap_or(1),
        )
    }
}

impl CfgGidMap {
    pub(crate) fn unwrap_or_default(self) -> (u32, u32, u32) {
        (
            self.container_id,
            self.host_id.unwrap_or(Gid::current().as_raw()),
            self.count.unwrap_or(1),
        )
    }
}
