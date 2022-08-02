use nix::{
    fcntl, fcntl::OFlag, mount, mount::MntFlags, mount::MsFlags, sched, sched::CloneFlags,
    sys::stat::Mode, unistd,
};
use std::{fs, path::Path};

use crate::{defer, tryfn, IDMap, Namespaces, Result};

const HOSTNAME: &str = "hakoniwa";
const NULL: Option<&'static Path> = None;

pub fn init(
    namespaces: &Namespaces,
    uid_mappings: &IDMap,
    gid_mappings: &IDMap,
    work_dir: &Path,
) -> Result<()> {
    let clone_flags = namespaces.to_clone_flags();
    tryfn!(sched::unshare(clone_flags), "unshare(...)")?;

    if clone_flags.contains(CloneFlags::CLONE_NEWUSER) {
        init_user_namespace(uid_mappings, gid_mappings)?;
    }
    if clone_flags.contains(CloneFlags::CLONE_NEWNS) {
        init_mount_namespace(work_dir)?;
    }
    if clone_flags.contains(CloneFlags::CLONE_NEWUTS) {
        init_uts_namespace()?;
    }

    Ok(())
}

fn init_user_namespace(uid_mappings: &IDMap, gid_mappings: &IDMap) -> Result<()> {
    write("/proc/self/uid_map", &format!("{}\n", uid_mappings))?;
    write("/proc/self/setgroups", "deny")?;
    write("/proc/self/gid_map", &format!("{}\n", gid_mappings))
}

fn init_mount_namespace(new_root: &Path) -> Result<()> {
    // Ensure that 'new_root' and its parent mount don't have
    // shared propagation (which would cause pivot_root() to
    // return an error), and prevent propagation of mount
    // events to the initial mount namespace.
    tryfn!(
        mount::mount(NULL, "/", NULL, MsFlags::MS_REC | MsFlags::MS_PRIVATE, NULL),
        "mount(NULL, {:?}, NULL, MS_REC | MS_PRIVATE, NULL)",
        "/"
    )?;

    // Ensure that 'new_root' is a mount point.
    tryfn!(
        mount::mount(Some(new_root), new_root, NULL, MsFlags::MS_BIND, NULL),
        "mount({:?}, {:?}, NULL, MS_BIND, NULL)",
        new_root,
        new_root
    )?;

    // Create directory to which old root will be pivoted.
    tryfn!(unistd::chdir(new_root), "chdir({:?})", new_root)?;
    tryfn!(
        unistd::mkdir(".put_old", Mode::S_IRWXU),
        "mkdir({:?}, S_IRWXU)",
        ".put_old"
    )?;

    // Pivot the root filesystem.
    tryfn!(
        unistd::pivot_root(".", ".put_old"),
        "pivot_root({:?}, {:?})",
        ".",
        ".put_old"
    )?;

    // Switch the current working directory to "/".
    tryfn!(unistd::chdir("/"), "chdir({:?})", "/")?;

    // Unmount old root and remove mount point.
    tryfn!(
        mount::umount2("/.put_old", MntFlags::MNT_DETACH),
        "umount2({:?}, MNT_DETACH)",
        "/.put_old"
    )?;
    tryfn!(fs::remove_dir("/.put_old"), "rmdir({:?})", "/.put_old")
}

fn init_uts_namespace() -> Result<()> {
    tryfn!(unistd::sethostname(HOSTNAME), "sethostname({:?})", HOSTNAME)
}

fn write(file: &str, content: &str) -> Result<()> {
    let fd = tryfn!(
        fcntl::open(file, OFlag::O_WRONLY, Mode::empty()),
        "open({:?}, O_WRONLY)",
        file
    )?;
    defer! { unistd::close(fd) }

    tryfn!(
        unistd::write(fd, content.as_bytes()),
        "write({:?}, ...)",
        file
    )?;
    Ok(())
}
