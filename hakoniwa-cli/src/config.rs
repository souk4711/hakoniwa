use anyhow::Result;
use minijinja::Environment;
use serde::Deserialize;
use std::env;
use std::fs;
use std::path::Path;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CfgConfig {
    #[serde(rename = "@include", default)]
    pub(crate) includes: Vec<String>,
    #[serde(rename = "namespaces", default)]
    pub(crate) namespaces: Vec<CfgNamespace>,
    #[serde(rename = "mounts", default)]
    pub(crate) mounts: Vec<CfgMount>,
    #[serde(rename = "envs", default)]
    pub(crate) envs: Vec<CfgEnv>,
    #[serde(rename = "rootdir")]
    pub(crate) rootdir: Option<CfgRootDir>,
    #[serde(rename = "uidmap")]
    pub(crate) uidmap: Option<CfgIdMap>,
    #[serde(rename = "gidmap")]
    pub(crate) gidmap: Option<CfgIdMap>,
    #[serde(rename = "hostname")]
    pub(crate) hostname: Option<String>,
    #[serde(rename = "network")]
    pub(crate) network: Option<CfgNetwork>,
    #[serde(rename = "limits", default)]
    pub(crate) limits: Vec<CfgLimit>,
    #[serde(rename = "landlock")]
    pub(crate) landlock: Option<CfgLandlock>,
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
    #[serde(rename = "envs", default)]
    pub(crate) envs: Vec<CfgEnv>,
    #[serde(rename = "landlock")]
    pub(crate) landlock: Option<CfgIncludeLandlock>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CfgIncludeLandlock {
    #[serde(rename = "fs")]
    pub(crate) fs: Vec<CfgLandlockFsRule>,
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
pub(crate) struct CfgEnv {
    #[serde(rename = "name")]
    pub(crate) name: String,
    #[serde(rename = "value")]
    pub(crate) value: Option<String>,
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
pub(crate) struct CfgIdMap {
    #[serde(rename = "container_id")]
    pub(crate) container_id: u32,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CfgNetwork {
    #[serde(rename = "mode")]
    pub(crate) mode: String,
    #[serde(rename = "options", default)]
    pub(crate) options: Vec<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CfgLimit {
    #[serde(rename = "type")]
    pub(crate) rtype: String,
    #[serde(rename = "value")]
    pub(crate) value: u64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CfgLandlock {
    #[serde(rename = "fs")]
    pub(crate) fs: Vec<CfgLandlockFsRule>,
}

#[derive(Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub(crate) struct CfgLandlockFsRule {
    #[serde(rename = "path")]
    pub(crate) path: String,
    #[serde(rename = "perm")]
    pub(crate) perm: String,
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
    // Similar to crate::contrib::clap::parse_setenv
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

pub(crate) fn load(path: &str) -> Result<CfgConfig> {
    // Template Renderer
    let mut r = Environment::empty();
    for (k, v) in env::vars() {
        r.add_global(k, v);
    }

    // CfgConfig
    log::debug!("CONFIG: {}", path);
    let path = fs::canonicalize(path)?;
    let data = fs::read_to_string(&path)?;

    // Parse CfgConfig
    let __dir__ = path.parent().unwrap_or(Path::new("/"));
    let data = r.render_str(&data, minijinja::context! { __dir__ })?;
    let mut config: CfgConfig = toml::from_str(&data)?;
    let mut cfgs = vec![];

    // Parse CfgInclude
    for include in &config.includes {
        let include = Path::new(&__dir__).join(include);
        log::debug!("CONFIG: Including {}", include.to_string_lossy());
        let path = fs::canonicalize(include)?;
        let data = fs::read_to_string(&path)?;

        let __dir__ = path.parent().unwrap_or(Path::new("/"));
        let data = r.render_str(&data, minijinja::context! { __dir__ })?;
        cfgs.push(toml::from_str::<CfgInclude>(&data)?);
    }

    // Merge Namespace, Mount, Env
    let mut namespaces = vec![];
    let mut mounts = vec![];
    let mut envs = vec![];
    for c in &cfgs {
        namespaces.extend(c.namespaces.clone());
        mounts.extend(c.mounts.clone());
        envs.extend(c.envs.clone());
    }
    namespaces.extend(config.namespaces);
    mounts.extend(config.mounts);
    envs.extend(config.envs);
    config.namespaces = namespaces;
    config.mounts = mounts;
    config.envs = envs;

    // Merge Landlock FS
    let mut fs = vec![];
    for c in cfgs {
        if let Some(landlock) = c.landlock {
            fs.extend(landlock.fs)
        }
    }
    if let Some(landlock) = &config.landlock {
        fs.extend(landlock.fs.clone());
    }
    if !fs.is_empty() {
        config.landlock = Some(CfgLandlock { fs });
    }

    // CfgConfig
    Ok(config)
}
