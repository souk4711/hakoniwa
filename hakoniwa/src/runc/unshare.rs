use crate::runc::error::*;
use crate::runc::nix::{self, CloneFlags};
use crate::{Container, Namespace};

macro_rules! ifthen {
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
    ifthen!(Namespace::Uts, container, sethostname)
}

fn sethostname(container: &Container) -> Result<()> {
    nix::sethostname(&container.hostname)
}

fn namespaces_to_clone_flags(container: &Container) -> CloneFlags {
    let mut flags = CloneFlags::empty();
    for flag in &container.namespaces {
        flags.insert(flag.to_clone_flag())
    }
    flags
}
