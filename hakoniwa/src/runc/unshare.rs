use crate::runc::error::*;
use crate::runc::nix::{self, CloneFlags};
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
    if_namespace_then!(Namespace::Uts, container, sethostname)?;
    Ok(())
}

pub(crate) fn unshare_remount_rootfs(container: &Container) -> Result<()> {
    if_namespace_then!(Namespace::User, container, setuidmap)?;
    if_namespace_then!(Namespace::User, container, setgidmap)?;
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
