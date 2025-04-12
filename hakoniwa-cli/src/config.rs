use anyhow::Result;
use minijinja::Environment;
use serde::Deserialize;
use std::env;
use std::fs;
use std::path::Path;

use crate::contrib::jinja;

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
    #[serde(rename = "filesystem")]
    pub(crate) filesystem: Option<CfgFileSystem>,
    #[serde(rename = "envs", default)]
    pub(crate) envs: Vec<CfgEnv>,
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
    #[serde(rename = "symlinks", default)]
    pub(crate) symlinks: Vec<CfgFileSystemSymlink>,
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
    r.add_test("dir", jinja::fs::is_dir);
    r.add_test("file", jinja::fs::is_file);
    r.add_test("symlink", jinja::fs::is_symlink);
    r.add_test("env", jinja::os::is_env);
    r.add_function("fs_glob", jinja::fs::glob);
    r.add_function("fs_read_link", jinja::fs::read_link);
    r.add_function("os_env", jinja::os::env);

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

    // Merge FileSystem, Landlock
    let mut landlock_created = false;
    let mut filesystem_created = false;
    let mut symlinks = vec![];
    let mut resources = vec![];
    let mut fs = vec![];
    let mut net = vec![];
    for c in cfgs {
        if let Some(landlock) = c.landlock {
            landlock_created = true;
            resources.extend(landlock.resources);
            fs.extend(landlock.fs);
            net.extend(landlock.net);
        }
        if let Some(filesystem) = c.filesystem {
            filesystem_created = true;
            symlinks.extend(filesystem.symlinks.clone());
        }
    }
    if let Some(landlock) = &config.landlock {
        landlock_created = true;
        resources.extend(landlock.resources.clone());
        fs.extend(landlock.fs.clone());
        net.extend(landlock.net.clone());
    }
    if let Some(filesystem) = &config.filesystem {
        filesystem_created = true;
        symlinks.extend(filesystem.symlinks.clone());
    }
    if landlock_created {
        config.landlock = Some(CfgLandlock { resources, fs, net });
    }
    if filesystem_created {
        config.filesystem = Some(CfgFileSystem { symlinks });
    }

    // CfgConfig
    Ok(config)
}
