use nix::{mount::MsFlags, sched::CloneFlags};
use std::path::Path;

use crate::{
    child_process::{error::Result, syscall},
    File, IDMap, Mount, Namespaces,
};

pub(crate) fn init(
    container_root_dir: &Path,
    namespaces: &Namespaces,
    hostname: &str,
    mounts: &[Mount],
) -> Result<()> {
    let clone_flags = namespaces.to_clone_flags();
    syscall::unshare(clone_flags)?;

    if clone_flags.contains(CloneFlags::CLONE_NEWNS) {
        init_mount_namespace(container_root_dir, mounts)?;
    }
    if clone_flags.contains(CloneFlags::CLONE_NEWUTS) {
        init_uts_namespace(hostname)?;
    }

    Ok(())
}

// [pivot_root]: https://man7.org/linux/man-pages/man2/pivot_root.2.html
fn init_mount_namespace(new_root: &Path, mounts: &[Mount]) -> Result<()> {
    // Ensure that "new_root" and its parent mount don't have
    // shared propagation (which would cause pivot_root() to
    // return an error), and prevent propagation of mount
    // events to the initial mount namespace.
    syscall::mount_root()?;

    // Ensure that "new_root" is a mount point.
    syscall::mount(new_root, new_root, MsFlags::MS_BIND)?;
    syscall::chdir(new_root)?;

    // Handle file system.
    {
        // Hang on to the old proc in order to mount the new proc later on.
        let target = new_root.join(Mount::PUT_OLD_PROC_DIR.0);
        syscall::mkdir_p(&target)?;
        syscall::mount("/proc", &target, MsFlags::MS_BIND | MsFlags::MS_REC)?;
        syscall::mkdir_p(new_root.join(Mount::PROC_DIR.0))?;

        // Handle user defined file system.
        for mount in mounts {
            let target = &mount.container_path.strip_prefix("/").unwrap_or_else(|_| {
                panic!(
                    "container_path({:?}) should start with a /",
                    mount.container_path
                )
            });

            match mount.fstype.as_deref() {
                None => {
                    let metadata = syscall::metadata(&mount.host_path)?;
                    match metadata.is_dir() {
                        true => syscall::mkdir_p(target)?,
                        _ => {
                            if let Some(dir) = target.parent() {
                                syscall::mkdir_p(dir)?;
                            }
                            syscall::touch(target)?
                        }
                    }
                    syscall::mount(
                        &mount.host_path,
                        target,
                        MsFlags::MS_BIND | MsFlags::MS_REC | mount.ms_flags(),
                    )?;
                }
                Some("tmpfs") => {
                    syscall::mkdir_p(target)?;
                    syscall::mount_tmpfs(target)?;
                }
                Some(fstype) => panic!(
                    "fstype({:?}) should be None or one of {:?}",
                    fstype,
                    ["tmpfs"]
                ),
            }
        }
    }

    // Create directory to which old root will be pivoted.
    syscall::mkdir_p(Mount::PUT_OLD_DIR.0)?;

    // Pivot the root filesystem.
    syscall::pivot_root(".", Mount::PUT_OLD_DIR.0)?;
    syscall::chdir("/")?;

    // Unmount old root and remove mount point.
    syscall::unmount(Mount::PUT_OLD_DIR.1)?;
    syscall::rmdir(Mount::PUT_OLD_DIR.1)
}

fn init_uts_namespace(hostname: &str) -> Result<()> {
    syscall::sethostname(hostname)
}

pub(crate) fn reinit(
    namespaces: &Namespaces,
    uid_mappings: &IDMap,
    gid_mappings: &IDMap,
    mounts: &[Mount],
    files: &[File],
    work_dir: &Path,
) -> Result<()> {
    let clone_flags = namespaces.to_clone_flags();

    if clone_flags.contains(CloneFlags::CLONE_NEWNS) {
        reinit_mount_namespace(mounts, files, work_dir)?;
    }
    if clone_flags.contains(CloneFlags::CLONE_NEWUSER) {
        reinit_user_namespace(uid_mappings, gid_mappings)?;
    }

    Ok(())
}

fn reinit_mount_namespace(mounts: &[Mount], files: &[File], work_dir: &Path) -> Result<()> {
    // Mount a new proc.
    syscall::mount_proc(Mount::PROC_DIR.1)?;
    syscall::unmount(Mount::PUT_OLD_PROC_DIR.1)?;
    syscall::rmdir(Mount::PUT_OLD_PROC_DIR.1)?;

    // Handle user defined file system.
    for mount in mounts {
        // Remount, make options read-write changed to read-only.
        if mount.fstype.is_none() && mount.ms_flags().contains(MsFlags::MS_RDONLY) {
            syscall::mount(
                &mount.container_path,
                &mount.container_path,
                MsFlags::MS_REMOUNT | MsFlags::MS_BIND | MsFlags::MS_REC | mount.ms_flags(),
            )?;
        }
    }

    // Create files
    for file in files {
        let target = &file.container_path;
        if let Some(dir) = target.parent() {
            syscall::mkdir_p(dir)?;
        }
        syscall::fwrite(target, &file.contents)?;
    }

    // Switch to the working directory.
    syscall::chdir(work_dir)?;
    Ok(())
}

fn reinit_user_namespace(uid_mappings: &IDMap, gid_mappings: &IDMap) -> Result<()> {
    syscall::fwrite("/proc/self/uid_map", &format!("{}\n", uid_mappings))?;
    syscall::fwrite("/proc/self/setgroups", "deny")?;
    syscall::fwrite("/proc/self/gid_map", &format!("{}\n", gid_mappings))
}
