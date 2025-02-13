use crate::runc::error::*;
use crate::runc::nix::{self, CloneFlags, MsFlags};
use crate::{Container, Namespace};

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
    nix::unshare(namespaces_to_clone_flags(container))?;
    if_namespace_then!(Namespace::Mount, container, mount_rootfs)?;
    if_namespace_then!(Namespace::Uts, container, sethostname)?;
    Ok(())
}

pub(crate) fn unshare_remount_rootfs(container: &Container) -> Result<()> {
    if_namespace_then!(Namespace::Mount, container, remount_rootfs)?;
    if_namespace_then!(Namespace::User, container, setuidmap)?;
    if_namespace_then!(Namespace::User, container, setgidmap)?;
    Ok(())
}

// [pivot_root]: https://man7.org/linux/man-pages/man2/pivot_root.2.html
fn mount_rootfs(container: &Container) -> Result<()> {
    // Get the mount point for the container root fs.
    let new_root = container.root_dir.as_path();

    // Ensure that "new_root" and its parent mount don't have
    // shared propagation (which would cause pivot_root() to
    // return an error), and prevent propagation of mount
    // events to the initial mount namespace.
    nix::mount_root()?;

    // Ensure that "new_root" is a mount point.
    nix::mount(new_root, new_root, MsFlags::MS_BIND)?;
    nix::chdir(new_root)?;

    // Hang on to the old proc in order to mount the new proc later on.
    let old_proc = new_root.join("oldproc");
    nix::mkdir_p(&old_proc)?;
    nix::mount("/proc", &old_proc, MsFlags::MS_BIND | MsFlags::MS_REC)?;
    nix::mkdir_p(new_root.join("proc"))?;

    // Create directory to which old root will be pivoted.
    nix::mkdir_p("oldrootfs")?;

    // Pivot the root filesystem.
    nix::pivot_root(".", "oldrootfs")?;

    // Switch the current working directory to "/".
    nix::chdir("/")?;

    // Unmount old root and remove mount point.
    nix::unmount("/oldrootfs")?;
    nix::rmdir("/oldrootfs")?;

    // Execute the command.
    // ...
    Ok(())
}

fn remount_rootfs(_container: &Container) -> Result<()> {
    // Mount a new proc.
    nix::mount_proc("/proc")?;
    nix::unmount("/oldproc")?;
    nix::rmdir("/oldproc")?;

    // ...
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
        nix::fwrite("/proc/self/gidmap", &format!("{}\n", gidmap))?;
        nix::fwrite("/proc/self/setgroups", "deny")
    } else {
        Ok(())
    }
}

fn namespaces_to_clone_flags(container: &Container) -> CloneFlags {
    let mut flags = CloneFlags::empty();
    for flag in &container.namespaces {
        flags.insert(flag.to_clone_flag())
    }
    flags
}
