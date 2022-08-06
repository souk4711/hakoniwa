use nix::{mount::MsFlags, sched::CloneFlags};
use std::path::{Path, PathBuf};

use crate::{IDMap, Mount, Namespaces, ResultWithError};

pub fn init(
    namespaces: &Namespaces,
    _uid_mappings: &IDMap,
    _gid_mappings: &IDMap,
    hostname: &str,
    rootfs: &Path,
    mounts: &[Mount],
    work_dir: &Path,
) -> ResultWithError<()> {
    let clone_flags = namespaces.to_clone_flags();
    super::syscall::unshare(clone_flags)?;

    if clone_flags.contains(CloneFlags::CLONE_NEWNS) {
        init_mount_namespace(rootfs, mounts, work_dir)?;
    }
    if clone_flags.contains(CloneFlags::CLONE_NEWUTS) {
        init_uts_namespace(hostname)?;
    }

    Ok(())
}

// [pivot_root]: https://man7.org/linux/man-pages/man2/pivot_root.2.html
fn init_mount_namespace(new_root: &Path, mounts: &[Mount], work_dir: &Path) -> ResultWithError<()> {
    // Ensure that 'new_root' and its parent mount don't have
    // shared propagation (which would cause pivot_root() to
    // return an error), and prevent propagation of mount
    // events to the initial mount namespace.
    super::syscall::mount_root()?;

    // Ensure that 'new_root' is a mount point.
    super::syscall::mount(new_root, new_root, MsFlags::MS_BIND)?;
    super::syscall::chdir(new_root)?;

    // Mount rootfs.
    {
        // Mount file system.
        for mount in mounts {
            let metadata = super::syscall::metadata(&mount.host_path)?;
            let target = &mount
                .container_path
                .strip_prefix("/")
                .unwrap_or(&mount.container_path);
            match metadata.is_dir() {
                true => super::syscall::mkdir(target)?,
                _ => super::syscall::touch(target)?,
            }
            super::syscall::mount(&mount.host_path, target, MsFlags::MS_BIND)?;
        }

        // Mount devfs.
        super::syscall::mkdir(new_root.join("dev"))?;
        for host_path in ["/dev/null", "/dev/random", "/dev/urandom", "/dev/zero"] {
            let target = host_path.strip_prefix('/').unwrap_or(host_path);
            super::syscall::mknod(&PathBuf::from(target))?;
            super::syscall::mount(host_path, target, MsFlags::MS_BIND)?;
        }

        // Hang on to the old proc in order to mount the new proc later on.
        let target = new_root.join(Mount::PUT_OLD_PROC_DIR.0);
        super::syscall::mkdir(&target)?;
        super::syscall::mount("/proc", &target, MsFlags::MS_BIND | MsFlags::MS_REC)?;
        super::syscall::mkdir(new_root.join(Mount::PROC_DIR.0))?;

        // Mount WORK_DIR.
        let target = new_root.join(Mount::WORK_DIR.0);
        super::syscall::mkdir(&target)?;
        super::syscall::mount(work_dir, &target, MsFlags::MS_BIND)?;
    }

    // Create directory to which old root will be pivoted.
    super::syscall::mkdir(Mount::PUT_OLD_DIR.0)?;

    // Pivot the root filesystem.
    super::syscall::pivot_root(".", Mount::PUT_OLD_DIR.0)?;
    super::syscall::chdir("/")?;

    // Unmount old root and remove mount point.
    super::syscall::unmount(Mount::PUT_OLD_DIR.1)?;
    super::syscall::rmdir(Mount::PUT_OLD_DIR.1)
}

fn init_uts_namespace(hostname: &str) -> ResultWithError<()> {
    super::syscall::sethostname(hostname)
}

pub fn reinit(
    namespaces: &Namespaces,
    uid_mappings: &IDMap,
    gid_mappings: &IDMap,
    mounts: &[Mount],
) -> ResultWithError<()> {
    let clone_flags = namespaces.to_clone_flags();

    if clone_flags.contains(CloneFlags::CLONE_NEWNS) {
        reinit_mount_namespace(mounts)?;
    }
    if clone_flags.contains(CloneFlags::CLONE_NEWUSER) {
        reinit_user_namespace(uid_mappings, gid_mappings)?;
    }

    Ok(())
}

fn reinit_mount_namespace(mounts: &[Mount]) -> ResultWithError<()> {
    // Remount read-only file system.
    for mount in mounts {
        let flags = MsFlags::MS_REMOUNT | MsFlags::MS_BIND | mount.ms_flags();
        super::syscall::mount(&mount.container_path, &mount.container_path, flags)?;
    }

    // Mount a new tmpfs.
    super::syscall::mkdir("/tmp")?;
    super::syscall::mount_tmpfs("/tmp")?;

    // Mount a new proc.
    super::syscall::mount_proc(Mount::PROC_DIR.1)?;
    super::syscall::unmount(Mount::PUT_OLD_PROC_DIR.1)?;
    super::syscall::rmdir(Mount::PUT_OLD_PROC_DIR.1)?;

    // Remount WORK_DIR as a read-write data volume.
    let flags = MsFlags::MS_REMOUNT | MsFlags::MS_BIND;
    super::syscall::mount(Mount::WORK_DIR.1, Mount::WORK_DIR.1, flags)?;

    // Switch to the working directory.
    super::syscall::chdir(Mount::WORK_DIR.1)
}

fn reinit_user_namespace(uid_mappings: &IDMap, gid_mappings: &IDMap) -> ResultWithError<()> {
    super::syscall::write("/proc/self/uid_map", &format!("{}\n", uid_mappings))?;
    super::syscall::write("/proc/self/setgroups", "deny")?;
    super::syscall::write("/proc/self/gid_map", &format!("{}\n", gid_mappings))
}
