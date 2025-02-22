use crate::runc::error::*;
use crate::runc::nix::{self, MsFlags, Path, PathBuf};
use crate::{Container, MountOptions, Namespace};

macro_rules! if_namespace_then {
    ($namespace:expr, $container:ident, $fn:ident) => {
        if $container.namespaces.contains(&$namespace) {
            $fn($container)
        } else {
            Ok(())
        }
    };
}

pub(crate) fn unshare(container: &Container) -> Result<()> {
    nix::unshare(container.get_namespaces_clone_flags())?;
    if_namespace_then!(Namespace::Mount, container, mount_rootfs)?;
    if_namespace_then!(Namespace::Uts, container, sethostname)?;
    Ok(())
}

pub(crate) fn tidyup(container: &Container) -> Result<()> {
    if_namespace_then!(Namespace::Mount, container, tidyup_rootfs)?;
    if_namespace_then!(Namespace::User, container, setuidmap)?;
    if_namespace_then!(Namespace::User, container, setgidmap)?;
    Ok(())
}

// [pivot_root]: https://man7.org/linux/man-pages/man2/pivot_root.2.html
fn mount_rootfs(container: &Container) -> Result<()> {
    // Get the mount point for the container root fs.
    let new_root = container.rootdir_abspath.as_path();

    // Ensure that "new_root" and its parent mount don't have
    // shared propagation (which would cause pivot_root() to
    // return an error), and prevent propagation of mount
    // events to the initial mount namespace.
    nix::mount_check_private("/")?;

    // Ensure that "new_root" is a mount point.
    nix::mount(new_root, new_root, MsFlags::MS_BIND)?;
    nix::chdir(new_root)?;

    // Mount all directories under "new_root".
    mount_rootfs_imp(container, new_root)?;

    // Create directory to which old root will be pivoted.
    nix::mkdir_p(".oldrootfs")?;

    // Pivot the root filesystem.
    nix::pivot_root(".", ".oldrootfs")?;

    // Switch the current working directory to "/".
    nix::chdir("/")?;

    // Unmount old root and remove mount point.
    nix::unmount("/.oldrootfs")?;
    nix::rmdir("/.oldrootfs")?;

    // Make options read-write changed to read-only.
    remount_rootfs_imp(container)?;

    // Execute the command.
    // ...
    Ok(())
}

fn mount_rootfs_imp(container: &Container, new_root: &Path) -> Result<()> {
    for mount in container.get_mounts() {
        let target_relpath = &mount
            .target
            .strip_prefix('/')
            .ok_or(Error::MountTargetPathMustBeAbsolute(mount.target.clone()))?;

        // Mount procfs.
        if mount.fstype == "proc" {
            if !container.namespaces.contains(&Namespace::Pid) {
                Err(Error::MountProcfsEPERM)?;
            }

            // Hang on to the old proc in order to mount the new proc later on.
            let old_proc = new_root.join(".oldproc");
            nix::mkdir_p(&old_proc)?;
            nix::mount("/proc", &old_proc, MsFlags::MS_BIND | MsFlags::MS_REC)?;
            nix::mkdir_p(new_root.join(target_relpath))?;
            continue;
        }

        // Mount tmpfs.
        if mount.fstype == "tmpfs" {
            nix::mkdir_p(target_relpath)?;
            nix::mount_filesystem(
                &mount.fstype,
                &mount.source,
                target_relpath,
                mount.options.to_ms_flags(),
            )?;
            continue;
        }

        // Mount devfs.
        if mount.fstype == "devfs" {
            nix::mkdir_p(target_relpath)?;
            // nix::mount_filesystem(
            //     "tmpfs",
            //     "tmpfs",
            //     target_relpath,
            //     mount.options.to_ms_flags(),
            // )?;
            mount_devfs_imp(target_relpath)?;
            continue;
        }

        // Mount unspecified filesystem type.
        let source_abspath = &mount.source;
        source_abspath
            .strip_prefix('/')
            .ok_or(Error::MountSourcePathMustBeAbsolute(source_abspath.clone()))?;
        let metadata = nix::metadata(source_abspath)?;
        if metadata.is_dir() {
            // - Directory
            nix::mkdir_p(target_relpath)?
        } else {
            // - Regular File
            // - Block/Character Device
            // - Socket
            PathBuf::from(&target_relpath).parent().map(nix::mkdir_p);
            nix::touch(target_relpath)?
        }
        nix::mount(source_abspath, target_relpath, mount.options.to_ms_flags())?;
    }
    Ok(())
}

fn mount_devfs_imp(target_relpath: &str) -> Result<()> {
    for dev in ["null", "zero", "full", "random", "urandom", "tty"] {
        let source = "/dev".to_string() + "/" + dev;
        let target = target_relpath.to_string() + "/" + dev;
        nix::touch(&target)?;
        nix::mount(source, target, MsFlags::MS_BIND | MsFlags::MS_NOSUID)?;
    }

    Ok(())
}

fn remount_rootfs_imp(container: &Container) -> Result<()> {
    for mount in container.get_mounts() {
        let target_relpath = &mount
            .target
            .strip_prefix('/')
            .ok_or(Error::MountTargetPathMustBeAbsolute(mount.target.clone()))?;

        let source_abspath = &mount.source;
        if mount.options.contains(MountOptions::RDONLY) {
            let mut options = mount.options.to_ms_flags();
            options.insert(MsFlags::MS_REMOUNT);
            nix::mount(source_abspath, target_relpath, options)?;
        }
    }
    Ok(())
}

fn tidyup_rootfs(container: &Container) -> Result<()> {
    // Mount a new proc.
    let mounts = container.get_mounts();
    let mount = mounts.iter().find(|mount| mount.fstype == "proc");
    if let Some(mount) = mount {
        nix::mount_filesystem(
            &mount.fstype,
            &mount.source,
            &mount.target,
            mount.options.to_ms_flags(),
        )?;
        nix::unmount("/.oldproc")?;
        nix::rmdir("/.oldproc")?;
    }
    Ok(())
}

fn sethostname(container: &Container) -> Result<()> {
    if let Some(hostname) = &container.hostname {
        nix::sethostname(hostname)
    } else {
        Ok(())
    }
}

fn setuidmap(container: &Container) -> Result<()> {
    if let Some(uidmap) = &container.uidmap {
        nix::fwrite("/proc/self/uid_map", &format!("{}\n", uidmap))
    } else {
        Ok(())
    }
}

fn setgidmap(container: &Container) -> Result<()> {
    if let Some(gidmap) = &container.gidmap {
        nix::fwrite("/proc/self/setgroups", "deny")?;
        nix::fwrite("/proc/self/gid_map", &format!("{}\n", gidmap))
    } else {
        Ok(())
    }
}
