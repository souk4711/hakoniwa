use landlock::*;

use crate::runc::error::*;
use crate::{landlock as ll, Container};

pub(crate) fn load(container: &Container) -> Result<()> {
    match &container.landlock_ruleset {
        Some(ruleset) => load_imp(ruleset),
        None => Ok(()),
    }
}

fn load_imp(ruleset: &ll::Ruleset) -> Result<()> {
    let abi = ABI::V5;
    let r = AccessFs::from_read(abi) & !AccessFs::Execute;
    let w = AccessFs::from_write(abi);
    let x = AccessFs::Execute;

    // Create a new landlock ruleset.
    let mut ctx = Ruleset::default()
        .set_compatibility(CompatLevel::HardRequirement)
        .handle_access(AccessFs::from_all(ABI::V1))?
        .set_compatibility(CompatLevel::BestEffort)
        .handle_access(AccessFs::from_all(abi))?
        .create()?;

    // Add fs rules.
    for rule in ruleset.get_fs_rules() {
        let mut mode = BitFlags::empty();
        for access in [ll::FsAccess::R, ll::FsAccess::W, ll::FsAccess::X] {
            if rule.mode & access == access {
                match access {
                    ll::FsAccess::R => mode |= r,
                    ll::FsAccess::W => mode |= w,
                    ll::FsAccess::X => mode |= x,
                    _ => unreachable!(),
                }
            }
        }
        let fd = PathFd::new(&rule.path)?;
        ctx = ctx.add_rule(PathBeneath::new(fd, mode))?;
    }

    // Load the ruleset.
    ctx.restrict_self()?;
    Ok(())
}
