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
        let mut access = BitFlags::empty();
        if rule.perm & ll::FsPerm::RD == ll::FsPerm::RD {
            access |= r;
        }
        if rule.perm & ll::FsPerm::WR == ll::FsPerm::WR {
            access |= w;
        }
        if rule.perm & ll::FsPerm::EXEC == ll::FsPerm::EXEC {
            access |= x;
        }

        let fd = PathFd::new(&rule.path)?;
        ctx = ctx.add_rule(PathBeneath::new(fd, access))?;
    }

    // Load the ruleset.
    ctx.restrict_self()?;
    Ok(())
}
