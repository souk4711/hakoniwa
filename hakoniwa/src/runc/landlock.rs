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
    // The Landlock ABI, should be incremented (and tested) regularly.
    let abi = ABI::V5;

    // Restrict FS.
    let mut ctx = Ruleset::default()
        .set_compatibility(CompatLevel::HardRequirement)
        .handle_access(AccessFs::from_all(ABI::V1))?
        .set_compatibility(CompatLevel::BestEffort)
        .handle_access(AccessFs::from_all(abi))?;

    // Restrict Net.
    for (resource, mode) in ruleset.restrictions.iter() {
        let compatibility = translate_compat_mode(*mode);
        let access = translate_resource(*resource);
        ctx = ctx.set_compatibility(compatibility).handle_access(access)?;
    }

    // Create a ruleset.
    let mut ctx = ctx.set_compatibility(CompatLevel::default()).create()?;

    // Add FS rules.
    let r = translate_fs_access(abi, ll::FsAccess::R);
    let w = translate_fs_access(abi, ll::FsAccess::W);
    let x = translate_fs_access(abi, ll::FsAccess::X);
    for rule in ruleset.get_fs_rules() {
        let mut access = BitFlags::empty();
        for e in [ll::FsAccess::R, ll::FsAccess::W, ll::FsAccess::X] {
            match rule.mode & e {
                ll::FsAccess::R => access |= r,
                ll::FsAccess::W => access |= w,
                ll::FsAccess::X => access |= x,
                _ => {}
            }
        }
        let path = std::fs::canonicalize(rule.path.clone())?;
        ctx = ctx.add_rules(path_beneath_rules(vec![path], access))?;
    }

    // Add NET rules.
    for (resource, _) in ruleset.restrictions.iter() {
        ctx = match resource {
            ll::Resource::NET_TCP_BIND => restrict_net(ctx, ruleset, resource)?,
            ll::Resource::NET_TCP_CONNECT => restrict_net(ctx, ruleset, resource)?,
        }
    }

    // Load the ruleset.
    ctx.restrict_self()?;
    Ok(())
}

fn restrict_net(
    mut ctx: RulesetCreated,
    ruleset: &ll::Ruleset,
    resource: &ll::Resource,
) -> Result<RulesetCreated> {
    let access = match resource {
        ll::Resource::NET_TCP_BIND => ll::NetAccess::TCP_BIND,
        ll::Resource::NET_TCP_CONNECT => ll::NetAccess::TCP_CONNECT,
    };
    if let Some(rules) = ruleset.net_rules.get(&access) {
        for e in rules {
            let rule = NetPort::new(e.port, translate_net_access(access));
            ctx = ctx.add_rule(rule)?;
        }
    }
    Ok(ctx)
}

fn translate_compat_mode(mode: ll::CompatMode) -> CompatLevel {
    match mode {
        ll::CompatMode::Enforce => CompatLevel::HardRequirement,
        ll::CompatMode::Relax => CompatLevel::BestEffort,
    }
}

fn translate_resource(resource: ll::Resource) -> impl Access {
    match resource {
        ll::Resource::NET_TCP_BIND => AccessNet::BindTcp,
        ll::Resource::NET_TCP_CONNECT => AccessNet::ConnectTcp,
    }
}

fn translate_fs_access(abi: ABI, access: ll::FsAccess) -> BitFlags<AccessFs> {
    match access {
        ll::FsAccess::R => AccessFs::from_read(abi) & !AccessFs::Execute,
        ll::FsAccess::W => AccessFs::from_write(abi),
        ll::FsAccess::X => AccessFs::Execute.into(),
        _ => unreachable!(),
    }
}

fn translate_net_access(access: ll::NetAccess) -> BitFlags<AccessNet> {
    match access {
        ll::NetAccess::TCP_BIND => AccessNet::BindTcp.into(),
        ll::NetAccess::TCP_CONNECT => AccessNet::ConnectTcp.into(),
        _ => unreachable!(),
    }
}
