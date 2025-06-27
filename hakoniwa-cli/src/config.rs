mod functions;
mod template;

use anyhow::Result;
use minijinja::Environment;
use std::fs;
use std::path::Path;

use crate::config::template::*;

pub(crate) fn load(path: &str) -> Result<CfgConfig> {
    // Template Renderer
    let mut r = Environment::new();
    r.add_function("fs_findup", functions::fs::findup);
    r.add_function("fs_glob", functions::fs::glob);
    r.add_function("fs_xdg_user_dir", functions::fs::xdg_user_dir);
    r.add_function("fs_mkdir", functions::fs::mkdir);
    r.add_function("fs_touch", functions::fs::touch);
    r.add_function("fs_read_link", functions::fs::read_link);
    r.add_function("os_env", functions::os::env);
    r.add_function("path_exists", functions::path::exists);
    r.add_function("path_is_dir", functions::path::is_dir);
    r.add_function("path_is_symlink", functions::path::is_symlink);

    // Template Renderer
    log::debug!("CONFIG: {path}");
    let path = fs::canonicalize(path)?;
    let data = fs::read_to_string(&path)?;
    let root = path.parent().unwrap_or(Path::new("/"));
    r.set_loader(minijinja::path_loader(root));

    // Parse CfgConfig
    let data = r.render_str(&data, minijinja::context! { __dir__ => root })?;
    let mut config: CfgConfig = toml::from_str(&data)?;

    // Parse CfgInclude
    let mut cfgs = vec![];
    for include in &config.includes {
        let include = Path::new(&root).join(include);
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

    // Merge Network
    let mut network = None;
    for c in &cfgs {
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
    for c in cfgs {
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
