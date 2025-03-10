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
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CfgNamespace {
    #[serde(rename = "type")]
    pub(crate) nstype: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CfgMount {
    #[serde(rename = "source")]
    pub(crate) source: String,
    #[serde(rename = "destination")]
    pub(crate) destination: String,
    #[serde(rename = "type", default)]
    pub(crate) fstype: String,
    #[serde(rename = "rw", default)]
    pub(crate) rw: bool,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CfgEnv {
    #[serde(rename = "name")]
    pub(crate) name: String,
    #[serde(rename = "value")]
    pub(crate) value: Option<String>,
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

    // Load CfgConfig
    let oldcwd = env::current_dir()?;
    log::trace!("CONFIG:     cwd: {}", oldcwd.to_string_lossy());
    log::debug!("CONFIG: loading: {}", path);
    let path = fs::canonicalize(path)?;
    let data = fs::read_to_string(&path)?;
    let data = r.render_str(&data, minijinja::context! {})?;
    let mut config: CfgConfig = toml::from_str(&data)?;
    let mut cfgs = vec![];

    // Load CfgInclude
    env::set_current_dir(path.parent().unwrap_or(Path::new("/")))?;
    log::trace!("CONFIG:     cwd: {}", env::current_dir()?.to_string_lossy());
    for include in &config.includes {
        log::debug!("CONFIG: loading: {}", include);
        let path = fs::canonicalize(include)?;
        let data = fs::read_to_string(path)?;
        let data = r.render_str(&data, minijinja::context! {})?;
        cfgs.push(toml::from_str::<CfgInclude>(&data)?);
    }
    env::set_current_dir(&oldcwd)?;
    log::trace!("CONFIG:     cwd: {}", oldcwd.to_string_lossy());

    // Merge CfgConfig & CfgInclude
    let mut namespaces = vec![];
    let mut mounts = vec![];
    let mut envs = vec![];
    for c in cfgs {
        namespaces.extend(c.namespaces);
        mounts.extend(c.mounts);
        envs.extend(c.envs);
    }
    namespaces.extend(config.namespaces);
    mounts.extend(config.mounts);
    envs.extend(config.envs);

    // CfgConfig
    config.namespaces = namespaces;
    config.mounts = mounts;
    config.envs = envs;
    Ok(config)
}
