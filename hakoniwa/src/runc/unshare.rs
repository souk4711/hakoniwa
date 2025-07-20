use super::error::*;
use super::sys::{self, FsFlags, MsFlags, PathBuf};
use crate::{Container, FsOperation, GroupFile, MountOptions, Namespace, PasswdFile, Runctl};

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
    if container.namespaces.is_empty() {
        return Ok(());
    }

    sys::unshare(container.get_namespaces_clone_flags())?;
    if_namespace_then!(Namespace::User, container, setuidmap)?;
    if_namespace_then!(Namespace::User, container, setgidmap)?;
    if_namespace_then!(Namespace::Mount, container, mount)?;
    Ok(())
}

pub(crate) fn tidyup(container: &Container) -> Result<()> {
    if container.namespaces.is_empty() {
        return Ok(());
    }

    if_namespace_then!(Namespace::Mount, container, mount2)?;
    if_namespace_then!(Namespace::Uts, container, sethostname)?;
    if_namespace_then!(Namespace::User, container, setuser)?;
    Ok(())
}

// [pivot_root]: https://man7.org/linux/man-pages/man2/pivot_root.2.html
fn mount(container: &Container) -> Result<()> {
    // Get the mount point for the container root fs.
    let new_root = container.rootdir_abspath.as_path();

    // Ensure that "new_root" and its parent mount don't have
    // shared propagation (which would cause pivot_root() to
    // return an error), and prevent propagation of mount
    // events to the initial mount namespace.
    sys::mount_make_private("/")?;

    // Ensure that "new_root" is a mount point.
    sys::mount(new_root, new_root, MsFlags::MS_BIND)?;

    // Initialize rootfs under "new_root".
    sys::chdir(new_root)?;
    initialize_rootfs(container)?;

    // Create directory to which "old_root" will be pivoted.
    sys::mkdir_p(".oldrootfs")?;

    // Pivot the root filesystem.
    sys::pivot_root(".", ".oldrootfs")?;

    // Switch the current working directory to "new_root".
    sys::chdir("/")?;

    // Unmount "old_root" and remove mount point.
    sys::unmount("/.oldrootfs")?;
    sys::rmdir("/.oldrootfs")?;

    // Make MsFlags::MS_RDONLY option work properly.
    remount_rdonly(container)?;

    // Apply filesystem operations.
    apply_fs_operations(container)?;

    // Done.
    Ok(())
}

// Initialize rootfs under Container#rootdir.
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
            sys::mkdir_p(".oldproc")?;
            sys::mount("/proc", ".oldproc", MsFlags::MS_BIND | MsFlags::MS_REC)?;
            sys::mkdir_p(target_relpath)?;
            continue;
        }

        // Mount tmpfs.
        if mount.fstype == "tmpfs" {
            sys::mkdir_p(target_relpath)?;
            sys::mount_filesystem(
                &mount.fstype,
                &mount.source,
                target_relpath,
                mount.options.to_ms_flags(),
            )?;
            continue;
        }

        // Mount devfs.
        if mount.fstype == "devfs" {
            sys::mkdir_p(target_relpath)?;
            sys::mount(
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
        let metadata = sys::metadata(source_abspath)?;
        if metadata.is_dir() {
            // - Directory
            sys::mkdir_p(target_relpath)?
        } else {
            // - Regular File
            // - Block/Character Device
            // - Socket
            PathBuf::from(&target_relpath).parent().map(sys::mkdir_p);
            sys::touch(target_relpath)?
        }
        sys::mount(source_abspath, target_relpath, mount.options.to_ms_flags())?;
    }
    Ok(())
}

// Initialize devfs under "target_relpath".
//
// [bubblewrap#SETUP_MOUNT_DEV]: https://github.com/containers/bubblewrap/blob/9ca3b05ec787acfb4b17bed37db5719fa777834f/bubblewrap.c#L1370
fn initialize_devfs(target_relpath: &str) -> Result<()> {
    for dev in ["null", "zero", "full", "random", "urandom", "tty"] {
        let source = format!("/dev/{dev}");
        let target = format!("{target_relpath}/{dev}");
        sys::touch(&target)?;
        sys::mount(source, target, MsFlags::MS_BIND | MsFlags::MS_NOSUID)?;
    }

    for (fd, dev) in ["stdin", "stdout", "stderr"].iter().enumerate() {
        let original = format!("/proc/self/fd/{fd}");
        let link = format!("{target_relpath}/{dev}");
        sys::symlink(original, link)?;
    }

    let fd_original = "/proc/self/fd".to_string();
    let fd_link = format!("{target_relpath}/fd");
    sys::symlink(fd_original, fd_link)?;

    let kcore_original = "/proc/kcore".to_string();
    let kcore_link = format!("{target_relpath}/core");
    sys::symlink(kcore_original, kcore_link)?;

    let shm_target_relpath = format!("{target_relpath}/shm");
    sys::mkdir_p(shm_target_relpath)?;

    let pts_target_relpath = format!("{target_relpath}/pts");
    let pts_flags = MsFlags::MS_NOSUID | MsFlags::MS_NOEXEC;
    sys::mkdir_p(&pts_target_relpath)?;
    sys::mount_filesystem("devpts", "devpts", pts_target_relpath, pts_flags)?;

    let ptmx_original = "pts/ptmx".to_string();
    let ptmx_link = format!("{target_relpath}/ptmx");
    sys::symlink(ptmx_original, ptmx_link)?;

    if sys::isatty()? {
        let source = sys::ttyname()?;
        let target = format!("{target_relpath}/console");
        let flags = MsFlags::MS_BIND | MsFlags::MS_NOSUID | MsFlags::MS_NOEXEC;
        sys::touch(&target)?;
        sys::mount(source, target, flags)?;
    }
    Ok(())
}

// Make MsFlags::MS_RDONLY option work properly.
fn remount_rdonly(container: &Container) -> Result<()> {
    for mount in container.get_mounts() {
        let target_relpath = &mount
            .target
            .strip_prefix('/')
            .ok_or(Error::MountTargetPathMustBeAbsolute(mount.target.clone()))?;

        if mount.options.contains(MountOptions::BIND) {
            let mut options = mount.options.to_ms_flags();
            options.insert(MsFlags::MS_REMOUNT);
            let res = sys::mount("", target_relpath, options);
            if res.is_ok() {
                continue;
            }

            if container.runctl.contains(&Runctl::MountFallback) {
                let options = unprivileged_mount_flags(target_relpath, options)?;
                sys::mount("", target_relpath, options)?;
            } else {
                res?;
            }
        }
    }
    Ok(())
}

// Apply filesystem operations.
fn apply_fs_operations(container: &Container) -> Result<()> {
    for op in &container.get_fs_operations() {
        match op {
            FsOperation::WriteFile(file) => {
                sys::fwrite(&file.target, &file.contents)?;
            }
            FsOperation::MakeDir(dir) => {
                sys::mkdir_p(&dir.target)?;
                sys::chmod(&dir.target, dir.mode)?;
            }
            FsOperation::MakeSymlink(symlink) => {
                sys::symlink(&symlink.original, &symlink.link)?;
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

    let stat = sys::statfs(path)?;
    for flag in stat.flags() {
        match flag {
            FsFlags::ST_RDONLY => flags.insert(MsFlags::MS_RDONLY),
            FsFlags::ST_NOSUID => flags.insert(MsFlags::MS_NOSUID),
            FsFlags::ST_NODEV => flags.insert(MsFlags::MS_NODEV),
            FsFlags::ST_NOEXEC => flags.insert(MsFlags::MS_NOEXEC),
            FsFlags::ST_NOATIME => flags.insert(MsFlags::MS_NOATIME),
            FsFlags::ST_NODIRATIME => flags.insert(MsFlags::MS_NODIRATIME),
            #[cfg(all(target_os = "linux", not(target_env = "musl")))]
            FsFlags::ST_RELATIME => flags.insert(MsFlags::MS_RELATIME),
            _ => {}
        }
    }
    Ok(flags)
}

// Mount procfs.
fn mount2(container: &Container) -> Result<()> {
    let mount = container.get_mount_newproc();
    if let Some(mount) = mount {
        sys::mount_filesystem(
            &mount.fstype,
            &mount.source,
            &mount.target,
            mount.options.to_ms_flags(),
        )?;
        sys::unmount("/.oldproc")?;
        sys::rmdir("/.oldproc")?;
    }

    if !container.runctl.contains(&Runctl::RootdirRW) {
        let mut options = MsFlags::MS_BIND | MsFlags::MS_REC | MsFlags::MS_REMOUNT;
        options = unprivileged_mount_flags(".", options)?;
        options.insert(MsFlags::MS_RDONLY);
        sys::mount("", ".", options)?;
    }

    Ok(())
}

// UID map to use for the user namespace.
fn setuidmap(container: &Container) -> Result<()> {
    if container.needs_mainp_setup_ugidmap() {
        return Ok(());
    }

    if let Some(uidmaps) = &container.uidmaps {
        sys::fwrite("/proc/self/uid_map", &uidmaps[0].to_line())
    } else {
        Ok(())
    }
}

// GID map to use for the user namespace.
fn setgidmap(container: &Container) -> Result<()> {
    if container.needs_mainp_setup_ugidmap() {
        return Ok(());
    }

    if let Some(gidmaps) = &container.gidmaps {
        sys::fwrite("/proc/self/setgroups", "deny")?;
        sys::fwrite("/proc/self/gid_map", &gidmaps[0].to_line())
    } else {
        Ok(())
    }
}

// Set the hostname in the container.
fn sethostname(container: &Container) -> Result<()> {
    if let Some(hostname) = &container.hostname {
        sys::sethostname(hostname)
    } else {
        Ok(())
    }
}

// Set the user/group in the container.
fn setuser(container: &Container) -> Result<()> {
    if container.user.is_some() {
        // In order to use the LANDLOCK or SECCOMP, either the calling thread
        // must have the CAP_SYS_ADMIN capability in its user namespace, or
        // the thread must allow to set no_new_privs bit.
        let nnp = !container.runctl.contains(&Runctl::AllowNewPrivs);
        sys::set_keepcaps(!nnp)?;

        // Set the user/group.
        let (uid, gid, sgids) = setuser_loadu(container)?;
        sys::setgroups(&sgids)?;
        sys::setgid(gid)?;
        sys::setuid(uid)
    } else {
        Ok(())
    }
}

// Get uid/gid/sgids.
fn setuser_loadu(container: &Container) -> Result<(u32, u32, Vec<u32>)> {
    let user = container.user.clone().expect("Container#user is some");
    let group = container.group.clone();
    let mut supplementary_groups = container.supplementary_groups.clone();
    let mut uid: u32 = u32::MAX;
    let mut gid: u32 = u32::MAX;
    let mut sgids = vec![];

    // Parse /etc/passwd, /etc/group files.
    let passwd_entries = PasswdFile::new("/etc/passwd").entries().map_err(|err| {
        let err = format!("/etc/passwd: {err}");
        Error::SetUserFailed(err)
    })?;
    let group_entries = GroupFile::new("/etc/group").entries().map_err(|err| {
        let err = format!("/etc/group: {err}");
        Error::SetUserFailed(err)
    })?;

    // Getuid.
    for entry in passwd_entries {
        if entry.name == user {
            uid = entry.uid;
            gid = entry.gid;
            break;
        }
    }
    if uid == u32::MAX {
        let err = "no matching entries in passwd file";
        let err = format!("unable to find user `{user}`: {err}");
        Err(Error::SetUserFailed(err))?;
    }

    // Getgid & Getgroups for default groups.
    if group.is_none() {
        for entry in group_entries {
            if entry.members.contains(&user) {
                sgids.push(entry.gid);
            }
        }
        return Ok((uid, gid, sgids));
    }

    // Getgid & Getgroups for specified groups.
    let mut gid: u32 = u32::MAX;
    let group = group.expect("Container::group is some");
    for entry in group_entries {
        if entry.name == group {
            gid = entry.gid;
        }
        if let Some(pos) = supplementary_groups.iter().position(|e| entry.name == *e) {
            sgids.push(entry.gid);
            supplementary_groups.remove(pos);
        }
    }
    if gid == u32::MAX {
        let err = "no matching entries in group file";
        let err = format!("unable to find group `{group}`: {err}");
        Err(Error::SetUserFailed(err))?;
    }
    if let Some(group) = supplementary_groups.into_iter().next() {
        let err = "no matching entries in group file";
        let err = format!("unable to find group `{group}`: {err}");
        Err(Error::SetUserFailed(err))?;
    }
    Ok((uid, gid, sgids))
}
