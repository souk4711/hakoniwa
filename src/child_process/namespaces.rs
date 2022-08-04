use nix::{mount::MsFlags, sched::CloneFlags};
use std::path::Path;

use crate::{IDMap, Mount, Namespaces, Result};

pub fn init(
    namespaces: &Namespaces,
    uid_mappings: &IDMap,
    gid_mappings: &IDMap,
    rootfs: &Path,
    mounts: &[Mount],
    work_dir: &Path,
) -> Result<()> {
    let clone_flags = namespaces.to_clone_flags();
    super::syscall::unshare(clone_flags)?;

    if clone_flags.contains(CloneFlags::CLONE_NEWUSER) {
        init_user_namespace(uid_mappings, gid_mappings)?;
    }
    if clone_flags.contains(CloneFlags::CLONE_NEWNS) {
        init_mount_namespace(rootfs, mounts, work_dir)?;
    }
    if clone_flags.contains(CloneFlags::CLONE_NEWUTS) {
        init_uts_namespace()?;
    }

    Ok(())
}

fn init_user_namespace(uid_mappings: &IDMap, gid_mappings: &IDMap) -> Result<()> {
    super::syscall::write("/proc/self/uid_map", &format!("{}\n", uid_mappings))?;
    super::syscall::write("/proc/self/setgroups", "deny")?;
    super::syscall::write("/proc/self/gid_map", &format!("{}\n", gid_mappings))
}

// [pivot_root]: https://man7.org/linux/man-pages/man2/pivot_root.2.html
fn init_mount_namespace(new_root: &Path, mounts: &[Mount], work_dir: &Path) -> Result<()> {
    // Ensure that 'new_root' and its parent mount don't have
    // shared propagation (which would cause pivot_root() to
    // return an error), and prevent propagation of mount
    // events to the initial mount namespace.
    super::syscall::mount_root()?;

    // Ensure that 'new_root' is a mount point.
    super::syscall::mount(new_root, new_root, MsFlags::MS_BIND)?;

    // Mount rootfs.
    {
        for mount in mounts {
            let target = &mount.target;
            super::syscall::mkdir(target)?;
            super::syscall::mount(&mount.source, target, MsFlags::MS_BIND)?;
        }

        let target = new_root.join(Mount::PUT_OLD_PROC_DIR.1);
        super::syscall::mkdir(&target)?;
        super::syscall::mount("/proc", &target, MsFlags::MS_BIND | MsFlags::MS_REC)?;
        super::syscall::mkdir(new_root.join(Mount::PROC_DIR.1))?;

        let target = new_root.join(Mount::WORK_DIR.1);
        super::syscall::mkdir(&target)?;
        super::syscall::mount(work_dir, &target, MsFlags::MS_BIND)?;
    }

    // Create directory to which old root will be pivoted.
    super::syscall::chdir(new_root)?;
    super::syscall::mkdir(Mount::PUT_OLD_DIR.1)?;

    // Pivot the root filesystem.
    super::syscall::pivot_root(".", Mount::PUT_OLD_DIR.1)?;
    super::syscall::chdir("/")?;

    // Unmount old root and remove mount point.
    super::syscall::unmount(Mount::PUT_OLD_DIR.0)?;
    super::syscall::rmdir(Mount::PUT_OLD_DIR.0)
}

fn init_uts_namespace() -> Result<()> {
    super::syscall::sethostname("hakoniwa")
}

pub fn reinit(namespaces: &Namespaces, mounts: &[Mount]) -> Result<()> {
    let clone_flags = namespaces.to_clone_flags();

    if clone_flags.contains(CloneFlags::CLONE_NEWNS) {
        reinit_mount_namespace(mounts)?;
    }

    Ok(())
}

fn reinit_mount_namespace(mounts: &[Mount]) -> Result<()> {
    // Mount read-only file system.
    for mount in mounts {
        let flags = MsFlags::MS_REMOUNT | MsFlags::MS_BIND | MsFlags::MS_RDONLY;
        super::syscall::mount(&mount.source, &mount.source, flags)?;
    }

    // Mount a new PROCFS.
    super::syscall::mount_proc(Mount::PROC_DIR.0)?;
    super::syscall::unmount(Mount::PUT_OLD_PROC_DIR.0)?;
    super::syscall::rmdir(Mount::PUT_OLD_PROC_DIR.0)?;

    // Mount WORK_DIR as a read-write data volume.
    let flags = MsFlags::MS_REMOUNT | MsFlags::MS_BIND;
    super::syscall::mount(Mount::WORK_DIR.0, Mount::WORK_DIR.0, flags)?;

    // Switch to the working directory.
    super::syscall::chdir(Mount::WORK_DIR.0)
}
