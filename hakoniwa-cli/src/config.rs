mod fields;
mod functions;
mod tests;

use anyhow::Result;
use minijinja::Environment;
use std::fs;
use std::path::Path;

use crate::config::fields::*;

pub(crate) fn load(path: &str) -> Result<CfgConfig> {
    // Template Renderer - functions
    let mut r = Environment::new();
    r.add_function("printenv", functions::printenv);
    r.add_function("mkdir_p", functions::mkdir_p);
    r.add_function("touch", functions::touch);
    r.add_function("find", functions::find);
    r.add_function("readlink", functions::readlink);
    r.add_function("xdg_user_dir", functions::xdg_user_dir);

    // Template Renderer - tests
    r.add_function("path_exists", tests::path_exists);
    r.add_function("path_is_dir", tests::path_is_dir);
    r.add_function("path_is_symlink", tests::path_is_symlink);

    // Template Renderer
    log::debug!("CONFIG: {path}");
    let path = fs::canonicalize(path)?;
    let data = fs::read_to_string(&path)?;
    let root = path.parent().expect("Path#parent is some");
    r.set_loader(minijinja::path_loader(root));

    // Parse CfgConfig
    let data = r.render_str(&data, minijinja::context! { __dir__ => root })?;
    let mut config: CfgConfig = toml::from_str(&data)?;

    // Parse CfgInclude
    let mut cfgincludes = vec![];
    for include in &config.includes {
        let include = Path::new(&root).join(include);
        log::debug!("CONFIG: Including {}", include.to_string_lossy());
        let path = fs::canonicalize(include)?;
        let data = fs::read_to_string(&path)?;

        let __dir__ = path.parent().expect("Path#parent is some");
        let data = r.render_str(&data, minijinja::context! { __dir__ })?;
        cfgincludes.push(toml::from_str::<CfgInclude>(&data)?);
    }

    // Merge Namespace, Mount, Env
    let mut namespaces = vec![];
    let mut mounts = vec![];
    let mut envs = vec![];
    for c in &cfgincludes {
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

    // Merge Network
    let mut network = None;
    for c in &cfgincludes {
        if c.network.is_some() {
            network = c.network.clone();
        }
    }
    if config.network.is_some() {
        network = config.network.clone();
    }
    config.network = network;

    // Merge FileSystem, Landlock
    let mut filesystem_created = false;
    let mut filesystem_files = vec![];
    let mut filesystem_dirs = vec![];
    let mut filesystem_symlinks = vec![];
    let mut landlock_created = false;
    let mut landlock_resources = vec![];
    let mut landlock_fs = vec![];
    let mut landlock_net = vec![];
    for c in cfgincludes {
        if let Some(filesystem) = c.filesystem {
            filesystem_created = true;
            filesystem_files.extend(filesystem.files.clone());
            filesystem_dirs.extend(filesystem.dirs.clone());
            filesystem_symlinks.extend(filesystem.symlinks.clone());
        }
        if let Some(landlock) = c.landlock {
            landlock_created = true;
            landlock_resources.extend(landlock.resources);
            landlock_fs.extend(landlock.fs);
            landlock_net.extend(landlock.net);
        }
    }
    if let Some(filesystem) = &config.filesystem {
        filesystem_created = true;
        filesystem_files.extend(filesystem.files.clone());
        filesystem_dirs.extend(filesystem.dirs.clone());
        filesystem_symlinks.extend(filesystem.symlinks.clone());
    }
    if let Some(landlock) = &config.landlock {
        landlock_created = true;
        landlock_resources.extend(landlock.resources.clone());
        landlock_fs.extend(landlock.fs.clone());
        landlock_net.extend(landlock.net.clone());
    }
    if filesystem_created {
        config.filesystem = Some(CfgFileSystem {
            files: filesystem_files,
            dirs: filesystem_dirs,
            symlinks: filesystem_symlinks,
        });
    }
    if landlock_created {
        config.landlock = Some(CfgLandlock {
            resources: landlock_resources,
            fs: landlock_fs,
            net: landlock_net,
        });
    }

    // CfgConfig
    Ok(config)
}
