use landlock::*;

use super::error::*;
use crate::{Container, Runctl, landlock as ll};

pub(crate) fn load(container: &Container) -> Result<()> {
    let nnp = !container.runctl.contains(&Runctl::AllowNewPrivs);
    match &container.landlock_ruleset {
        Some(ruleset) => load_imp(ruleset, nnp),
        None => Ok(()),
    }
}

fn load_imp(ruleset: &ll::Ruleset, nnp: bool) -> Result<()> {
    if ruleset.restrictions.is_empty() {
        return Ok(());
    }

    let abi = ABI::V5;
    let mut ctx = Ruleset::default();

    for (resource, mode) in ruleset.restrictions.iter() {
        ctx = match resource {
            ll::Resource::FS => handle_access_fs(ctx, abi)?,
            ll::Resource::NET_TCP_BIND => handle_access_net(ctx, resource, mode)?,
            ll::Resource::NET_TCP_CONNECT => handle_access_net(ctx, resource, mode)?,
        }
    }

    let mut ctx = ctx.set_compatibility(CompatLevel::default()).create()?;
    for (resource, _) in ruleset.restrictions.iter() {
        ctx = match resource {
            ll::Resource::FS => add_rules_fs(ctx, abi, ruleset)?,
            ll::Resource::NET_TCP_BIND => add_rules_net(ctx, ruleset, resource)?,
            ll::Resource::NET_TCP_CONNECT => add_rules_net(ctx, ruleset, resource)?,
        }
    }

    ctx = ctx.set_no_new_privs(nnp);
    ctx.restrict_self()?;
    Ok(())
}

fn handle_access_fs(mut ctx: Ruleset, abi: ABI) -> Result<Ruleset> {
    ctx = ctx
        .set_compatibility(CompatLevel::HardRequirement)
        .handle_access(AccessFs::from_all(ABI::V1))
        .map_err(|e| translate_landlock_ruleset_error(ll::Resource::FS, e))?
        .set_compatibility(CompatLevel::BestEffort)
        .handle_access(AccessFs::from_all(abi))?;
    Ok(ctx)
}

fn handle_access_net(
    mut ctx: Ruleset,
    resource: &ll::Resource,
    mode: &ll::CompatMode,
) -> Result<Ruleset> {
    let compatibility = translate_compat_mode(*mode);
    let access = translate_net_resource(*resource);
    ctx = ctx
        .set_compatibility(compatibility)
        .handle_access(access)
        .map_err(|e| translate_landlock_ruleset_error(*resource, e))?;
    Ok(ctx)
}

fn add_rules_fs(
    mut ctx: RulesetCreated,
    abi: ABI,
    ruleset: &ll::Ruleset,
) -> Result<RulesetCreated> {
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
        let path = std::fs::canonicalize(rule.path.clone())
            .map_err(|_| Error::LandlockPathMustBeAbsolute(rule.path.clone()))?;
        ctx = ctx.add_rules(path_beneath_rules(vec![path], access))?;
    }
    Ok(ctx)
}

fn add_rules_net(
    mut ctx: RulesetCreated,
    ruleset: &ll::Ruleset,
    resource: &ll::Resource,
) -> Result<RulesetCreated> {
    if let Some(rules) = ruleset.net_rules.get(resource) {
        for e in rules {
            let rule = NetPort::new(e.port, translate_net_access(e.access));
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

fn translate_net_resource(resource: ll::Resource) -> AccessNet {
    match resource {
        ll::Resource::NET_TCP_BIND => AccessNet::BindTcp,
        ll::Resource::NET_TCP_CONNECT => AccessNet::ConnectTcp,
        _ => unreachable!(),
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

fn translate_landlock_ruleset_error(resource: ll::Resource, e: landlock::RulesetError) -> Error {
    // [landlock#VERSIONS]: https://man7.org/linux/man-pages/man7/landlock.7.html#VERSIONS
    let (f, m) = match resource {
        ll::Resource::FS => ("Filesystem restrictions", "5.13"),
        ll::Resource::NET_TCP_BIND => ("Network TCP restrictions", "6.7"),
        ll::Resource::NET_TCP_CONNECT => ("Network TCP restrictions", "6.7"),
    };
    Error::LandlockFeatureUnsupported(f.to_string(), m.to_string(), e.to_string())
}
