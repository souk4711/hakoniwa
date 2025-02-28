use crate::runc::error::*;
use crate::runc::nix::{self, FsFlags, MsFlags, PathBuf};
use crate::{Container, MountOptions, Namespace, Runctl};

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
    if_namespace_then!(Namespace::User, container, setuidmap)?;
    if_namespace_then!(Namespace::User, container, setgidmap)?;
    if_namespace_then!(Namespace::Mount, container, mount_rootfs)?;
    if_namespace_then!(Namespace::Uts, container, sethostname)?;
    Ok(())
}

pub(crate) fn tidyup(container: &Container) -> Result<()> {
    if_namespace_then!(Namespace::Mount, container, tidyup_rootfs)?;
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
    nix::mount_make_private("/")?;

    // Ensure that "new_root" is a mount point.
    nix::mount(new_root, new_root, MsFlags::MS_BIND)?;

    // Initialize rootfs under "new_root".
    nix::chdir(new_root)?;
    initialize_rootfs(container)?;

    // Create directory to which "old_root" will be pivoted.
    nix::mkdir_p(".oldrootfs")?;

    // Pivot the root filesystem.
    nix::pivot_root(".", ".oldrootfs")?;

    // Switch the current working directory to "new_root".
    nix::chdir("/")?;

    // Unmount "old_root" and remove mount point.
    nix::unmount("/.oldrootfs")?;
    nix::rmdir("/.oldrootfs")?;

    // Make options read-write changed to read-only.
    remount_rootfs_rdonly(container)?;

    // Fork...
    // ...
    Ok(())
}

fn initialize_rootfs(container: &Container) -> Result<()> {
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
            nix::mkdir_p(".oldproc")?;
            nix::mount("/proc", ".oldproc", MsFlags::MS_BIND | MsFlags::MS_REC)?;
            nix::mkdir_p(target_relpath)?;
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
            nix::mount(
                target_relpath,
                target_relpath,
                MsFlags::MS_BIND | MsFlags::MS_NOSUID,
            )?;
            initialize_devfs(target_relpath)?;
            continue;
        }

        // Bind Mounts.
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

// [bubblewrap#SETUP_MOUNT_DEV]: https://github.com/containers/bubblewrap/blob/9ca3b05ec787acfb4b17bed37db5719fa777834f/bubblewrap.c#L1370
fn initialize_devfs(target_relpath: &str) -> Result<()> {
    for dev in ["null", "zero", "full", "random", "urandom", "tty"] {
        let source = format!("/dev/{}", dev);
        let target = format!("{}/{}", target_relpath, dev);
        nix::touch(&target)?;
        nix::mount(source, target, MsFlags::MS_BIND | MsFlags::MS_NOSUID)?;
    }

    for (fd, dev) in ["stdin", "stdout", "stderr"].iter().enumerate() {
        let original = format!("/proc/self/fd/{}", fd);
        let link = format!("{}/{}", target_relpath, dev);
        nix::symlink(original, link)?;
    }

    let shm_target_relpath = format!("{}/shm", target_relpath);
    nix::mkdir_p(shm_target_relpath)?;

    let pts_target_relpath = format!("{}/pts", target_relpath);
    let pts_flags = MsFlags::MS_NOSUID | MsFlags::MS_NOEXEC;
    nix::mkdir_p(&pts_target_relpath)?;
    nix::mount_filesystem("devpts", "devpts", pts_target_relpath, pts_flags)?;

    let ptmx_original = format!("/{}/pts/ptmx", target_relpath);
    let ptmx_link = format!("{}/ptmx", target_relpath);
    nix::symlink(ptmx_original, ptmx_link)?;

    if nix::isatty()? {
        let source = nix::ttyname()?;
        let target = format!("{}/console", target_relpath);
        let flags = MsFlags::MS_BIND | MsFlags::MS_NOSUID | MsFlags::MS_NOEXEC;
        nix::touch(&target)?;
        nix::mount(source, target, flags)?;
    }
    Ok(())
}

fn remount_rootfs_rdonly(container: &Container) -> Result<()> {
    for mount in container.get_mounts() {
        let target_relpath = &mount
            .target
            .strip_prefix('/')
            .ok_or(Error::MountTargetPathMustBeAbsolute(mount.target.clone()))?;

        if mount.options.contains(MountOptions::BIND) {
            let mut options = mount.options.to_ms_flags();
            options.insert(MsFlags::MS_REMOUNT);
            let res = nix::mount("", target_relpath, options);
            if res.is_ok() {
                continue;
            }

            if container.runctl.contains(&Runctl::MountFallback) {
                let options = unprivileged_mount_flags(target_relpath, options)?;
                nix::mount("", target_relpath, options)?;
            } else {
                res?;
            }
        }
    }
    Ok(())
}

// Get the set of mount flags that are set on the mount that contains the given
// path and are locked by CL_UNPRIVILEGED. This is necessary to ensure that
// bind-mounting "with options" will not fail with user namespaces, due to
// kernel restrictions that require user namespace mounts to preserve
// CL_UNPRIVILEGED locked flags.
//
// [moby#getUnprivilegedMountFlags]: https://github.com/moby/moby/blob/94d3ad69cc598b5a8eb8a643d6999375953512e8/daemon/oci_linux.go#L435
fn unprivileged_mount_flags(path: &str, mut flags: MsFlags) -> Result<MsFlags> {
    for flag in [
        MsFlags::MS_RDONLY,
        MsFlags::MS_NOSUID,
        MsFlags::MS_NODEV,
        MsFlags::MS_NOEXEC,
        MsFlags::MS_NOATIME,
        MsFlags::MS_NODIRATIME,
        MsFlags::MS_RELATIME,
    ] {
        flags.remove(flag);
    }

    let stat = nix::statfs(path)?;
    for flag in stat.flags() {
        match flag {
            FsFlags::ST_RDONLY => flags.insert(MsFlags::MS_RDONLY),
            FsFlags::ST_NOSUID => flags.insert(MsFlags::MS_NOSUID),
            FsFlags::ST_NODEV => flags.insert(MsFlags::MS_NODEV),
            FsFlags::ST_NOEXEC => flags.insert(MsFlags::MS_NOEXEC),
            FsFlags::ST_NOATIME => flags.insert(MsFlags::MS_NOATIME),
            FsFlags::ST_NODIRATIME => flags.insert(MsFlags::MS_NODIRATIME),
            FsFlags::ST_RELATIME => flags.insert(MsFlags::MS_RELATIME),
            _ => {}
        }
    }
    Ok(flags)
}

fn tidyup_rootfs(container: &Container) -> Result<()> {
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

    if !container.runctl.contains(&Runctl::RootdirRW) {
        let mut options = MsFlags::MS_BIND | MsFlags::MS_REC | MsFlags::MS_REMOUNT;
        options = unprivileged_mount_flags(".", options)?;
        options.insert(MsFlags::MS_RDONLY);
        nix::mount("", ".", options)?;
    }

    Ok(())
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

fn sethostname(container: &Container) -> Result<()> {
    if let Some(hostname) = &container.hostname {
        nix::sethostname(hostname)
    } else {
        Ok(())
    }
}
