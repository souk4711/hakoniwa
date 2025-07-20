use libseccomp::*;

use super::{error::*, sys};
use crate::{Container, Runctl, seccomp::*};

pub(crate) fn load(container: &Container) -> Result<()> {
    let nnp = !container.runctl.contains(&Runctl::AllowNewPrivs);
    match &container.seccomp_filter {
        Some(filter) => load_imp(filter, nnp),
        None => match nnp {
            true => sys::set_no_new_privs(),
            _ => Ok(()),
        },
    }
}

fn load_imp(filter: &Filter, nnp: bool) -> Result<()> {
    // Create a new filter context.
    let default_scmp_action = translate_action(filter.default_action);
    let mut ctx = ScmpFilterContext::new(default_scmp_action)?;
    ctx.set_ctl_nnp(nnp)?;

    // Add architectures.
    for arch in &filter.architectures {
        let scmp_arch = translate_arch(*arch);
        ctx.add_arch(scmp_arch)?;
    }

    // Add rules.
    for rule in &filter.rules {
        let (action, sysname, argcmps) = (rule.action, &rule.sysname, &rule.argcmps);

        // If the action is the same as the default action, the rule is
        // redundant, skip it.
        let scmp_action = translate_action(action);
        if scmp_action == default_scmp_action {
            continue;
        }

        // If the syscall is not supported by the kernel, skip it.
        let scmp_syscall = match ScmpSyscall::from_name(sysname) {
            Ok(syscall) => syscall,
            Err(_) => continue,
        };

        // Adds a single rule for an unconditional action on a syscall.
        if argcmps.is_empty() {
            ctx.add_rule(scmp_action, scmp_syscall)?;
            continue;
        }

        // Adds a single rule for a conditional action on a syscall.
        let scmp_argcmps = translate_argcmps(argcmps);
        ctx.add_rule_conditional(scmp_action, scmp_syscall, &scmp_argcmps)?;
    }

    // Load the filter.
    Ok(ctx.load()?)
}

fn translate_action(action: Action) -> ScmpAction {
    match action {
        Action::Allow => ScmpAction::Allow,
        Action::Errno(v) => ScmpAction::Errno(v),
        Action::KillProcess => ScmpAction::KillProcess,
        Action::KillThread => ScmpAction::KillThread,
        Action::Log => ScmpAction::Log,
        Action::Notify => ScmpAction::Notify,
        Action::Trace(v) => ScmpAction::Trace(v),
        Action::Trap => ScmpAction::Trap,
    }
}

fn translate_arch(arch: Arch) -> ScmpArch {
    match arch {
        Arch::Native => ScmpArch::Native,
        Arch::X86 => ScmpArch::X86,
        Arch::X8664 => ScmpArch::X8664,
        Arch::X32 => ScmpArch::X32,
        Arch::Arm => ScmpArch::Arm,
        Arch::Aarch64 => ScmpArch::Aarch64,
        Arch::Loongarch64 => ScmpArch::Loongarch64,
        Arch::M68k => ScmpArch::M68k,
        Arch::Mips => ScmpArch::Mips,
        Arch::Mips64 => ScmpArch::Mips64,
        Arch::Mips64n32 => ScmpArch::Mips64N32,
        Arch::Mipsel => ScmpArch::Mipsel,
        Arch::Mipsel64 => ScmpArch::Mipsel64,
        Arch::Mipsel64n32 => ScmpArch::Mipsel64N32,
        Arch::Ppc => ScmpArch::Ppc,
        Arch::Ppc64 => ScmpArch::Ppc64,
        Arch::Ppc64le => ScmpArch::Ppc64Le,
        Arch::S390 => ScmpArch::S390,
        Arch::S390x => ScmpArch::S390X,
        Arch::Parisc => ScmpArch::Parisc,
        Arch::Parisc64 => ScmpArch::Parisc64,
        Arch::Riscv64 => ScmpArch::Riscv64,
        Arch::Sheb => ScmpArch::Sheb,
        Arch::Sh => ScmpArch::Sh,
    }
}

fn translate_argcmps(argcmps: &[ArgCmp]) -> Vec<ScmpArgCompare> {
    argcmps
        .iter()
        .map(|cmp| {
            let mut datum = cmp.datum_a;
            let op = match cmp.op {
                ArgCmpOp::Ne => ScmpCompareOp::NotEqual,
                ArgCmpOp::Lt => ScmpCompareOp::Less,
                ArgCmpOp::Le => ScmpCompareOp::LessOrEqual,
                ArgCmpOp::Eq => ScmpCompareOp::Equal,
                ArgCmpOp::Gt => ScmpCompareOp::Greater,
                ArgCmpOp::Ge => ScmpCompareOp::GreaterEqual,
                ArgCmpOp::MaskedEq => {
                    datum = cmp.datum_b;
                    ScmpCompareOp::MaskedEqual(cmp.datum_a)
                }
            };
            ScmpArgCompare::new(cmp.arg, op, datum)
        })
        .collect()
}
