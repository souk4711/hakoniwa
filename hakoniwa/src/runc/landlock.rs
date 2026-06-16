use landlock::{
    ABI as LandlockABI, Access as LandlockAccess, AccessFs as LandlockAccessFs,
    AccessNet as LandlockAccessNet, BitFlags as LandlockBitFlags,
    CompatLevel as LandlockCompatLevel, Compatible as LandlockCompatible,
    NetPort as LandlockNetPort, Ruleset as LandlockRuleset, RulesetAttr as LandlockRulesetAttr,
    RulesetCreated as LandlockRulesetCreated, RulesetCreatedAttr as LandlockRulesetCreatedAttr,
    RulesetError as LandlockRulesetError, path_beneath_rules as landlock_path_beneath_rules,
};

use super::error::*;
use crate::{Container, Runctl};

pub(crate) fn load(container: &Container) -> Result<()> {
    let nnp = !container.runctl.contains(&Runctl::AllowNewPrivs);
    match &container.landlock_ruleset {
        Some(ruleset) => load_imp(ruleset, nnp),
        None => Ok(()),
    }
}

fn load_imp(ruleset: &crate::landlock::Ruleset, nnp: bool) -> Result<()> {
    if ruleset.restrictions.is_empty() {
        return Ok(());
    }

    let abi = LandlockABI::V5;
    let mut ctx = LandlockRuleset::default();

    for (resource, mode) in ruleset.restrictions.iter() {
        ctx = match resource {
            crate::landlock::Resource::FS => handle_access_fs(ctx, abi)?,
            crate::landlock::Resource::NET_TCP_BIND => handle_access_net(ctx, resource, mode)?,
            crate::landlock::Resource::NET_TCP_CONNECT => handle_access_net(ctx, resource, mode)?,
        }
    }

    let mut ctx = ctx
        .set_compatibility(LandlockCompatLevel::default())
        .create()?;
    for (resource, _) in ruleset.restrictions.iter() {
        ctx = match resource {
            crate::landlock::Resource::FS => add_rules_fs(ctx, abi, ruleset)?,
            crate::landlock::Resource::NET_TCP_BIND => add_rules_net(ctx, ruleset, resource)?,
            crate::landlock::Resource::NET_TCP_CONNECT => add_rules_net(ctx, ruleset, resource)?,
        }
    }

    ctx = ctx.no_new_privs(nnp);
    ctx.restrict_self()?;
    Ok(())
}

fn handle_access_fs(mut ctx: LandlockRuleset, abi: LandlockABI) -> Result<LandlockRuleset> {
    ctx = ctx
        .set_compatibility(LandlockCompatLevel::HardRequirement)
        .handle_access(LandlockAccessFs::from_all(LandlockABI::V1))
        .map_err(|e| translate_landlock_ruleset_error(crate::landlock::Resource::FS, e))?
        .set_compatibility(LandlockCompatLevel::BestEffort)
        .handle_access(LandlockAccessFs::from_all(abi))?;
    Ok(ctx)
}

fn handle_access_net(
    mut ctx: LandlockRuleset,
    resource: &crate::landlock::Resource,
    mode: &crate::landlock::CompatMode,
) -> Result<LandlockRuleset> {
    let compatibility = translate_compat_mode(*mode);
    let access = translate_net_resource(*resource);
    ctx = ctx
        .set_compatibility(compatibility)
        .handle_access(access)
        .map_err(|e| translate_landlock_ruleset_error(*resource, e))?;
    Ok(ctx)
}

fn add_rules_fs(
    mut ctx: LandlockRulesetCreated,
    abi: LandlockABI,
    ruleset: &crate::landlock::Ruleset,
) -> Result<LandlockRulesetCreated> {
    let r = translate_fs_access(abi, crate::landlock::FsAccess::R);
    let w = translate_fs_access(abi, crate::landlock::FsAccess::W);
    let x = translate_fs_access(abi, crate::landlock::FsAccess::X);
    for rule in ruleset.get_fs_rules() {
        let mut access = LandlockBitFlags::empty();
        for e in [
            crate::landlock::FsAccess::R,
            crate::landlock::FsAccess::W,
            crate::landlock::FsAccess::X,
        ] {
            match rule.mode & e {
                crate::landlock::FsAccess::R => access |= r,
                crate::landlock::FsAccess::W => access |= w,
                crate::landlock::FsAccess::X => access |= x,
                _ => {}
            }
        }
        let path = std::fs::canonicalize(rule.path.clone())
            .map_err(|_| Error::LandlockPathMustBeAbsolute(rule.path.clone()))?;
        ctx = ctx.add_rules(landlock_path_beneath_rules([path], access))?;
    }
    Ok(ctx)
}

fn add_rules_net(
    mut ctx: LandlockRulesetCreated,
    ruleset: &crate::landlock::Ruleset,
    resource: &crate::landlock::Resource,
) -> Result<LandlockRulesetCreated> {
    if let Some(rules) = ruleset.net_rules.get(resource) {
        for e in rules {
            let rule = LandlockNetPort::new(e.port, translate_net_access(e.access));
            ctx = ctx.add_rule(rule)?;
        }
    }
    Ok(ctx)
}

fn translate_compat_mode(mode: crate::landlock::CompatMode) -> LandlockCompatLevel {
    match mode {
        crate::landlock::CompatMode::Enforce => LandlockCompatLevel::HardRequirement,
        crate::landlock::CompatMode::Relax => LandlockCompatLevel::BestEffort,
    }
}

fn translate_net_resource(resource: crate::landlock::Resource) -> LandlockAccessNet {
    match resource {
        crate::landlock::Resource::NET_TCP_BIND => LandlockAccessNet::BindTcp,
        crate::landlock::Resource::NET_TCP_CONNECT => LandlockAccessNet::ConnectTcp,
        _ => unreachable!("runc::landlock::translate_net_resource"),
    }
}

fn translate_fs_access(
    abi: LandlockABI,
    access: crate::landlock::FsAccess,
) -> LandlockBitFlags<LandlockAccessFs> {
    match access {
        crate::landlock::FsAccess::R => {
            LandlockAccessFs::from_read(abi) & !LandlockAccessFs::Execute
        }
        crate::landlock::FsAccess::W => LandlockAccessFs::from_write(abi),
        crate::landlock::FsAccess::X => LandlockAccessFs::Execute.into(),
        _ => unreachable!("runc::landlock::translate_fs_access"),
    }
}

fn translate_net_access(access: crate::landlock::NetAccess) -> LandlockBitFlags<LandlockAccessNet> {
    match access {
        crate::landlock::NetAccess::TCP_BIND => LandlockAccessNet::BindTcp.into(),
        crate::landlock::NetAccess::TCP_CONNECT => LandlockAccessNet::ConnectTcp.into(),
        _ => unreachable!("runc::landlock::translate_net_access"),
    }
}

fn translate_landlock_ruleset_error(
    resource: crate::landlock::Resource,
    e: LandlockRulesetError,
) -> Error {
    // [landlock#VERSIONS]: https://man7.org/linux/man-pages/man7/landlock.7.html#VERSIONS
    let (f, m) = match resource {
        crate::landlock::Resource::FS => ("Filesystem restrictions", "5.13"),
        crate::landlock::Resource::NET_TCP_BIND => ("Network TCP restrictions", "6.7"),
        crate::landlock::Resource::NET_TCP_CONNECT => ("Network TCP restrictions", "6.7"),
    };
    Error::LandlockFeatureUnsupported(f.to_string(), m.to_string(), e.to_string())
}
