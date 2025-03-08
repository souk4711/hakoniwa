use anyhow::{anyhow, Result};
use serde::Deserialize;
use std::str::FromStr;

use crate::assets::Assets;
use hakoniwa::seccomp::*;

#[derive(Deserialize)]
struct CfgSeccomp {
    #[serde(rename = "defaultAction")]
    default_action: String,
    #[serde(rename = "defaultErrnoRet", default)]
    default_errno_ret: i32,
    #[serde(rename = "archMap", default)]
    arch_map: Vec<CfgArchitecture>,
    #[serde(rename = "syscalls", default)]
    syscalls: Vec<CfgSyscall>,
}

#[derive(Deserialize)]
struct CfgArchitecture {
    #[serde(rename = "architecture")]
    arch: String,
    #[serde(rename = "subArchitectures", default)]
    sub_arches: Vec<String>,
}

#[derive(Deserialize)]
struct CfgSyscall {
    #[serde(rename = "names")]
    names: Vec<String>,
    #[serde(rename = "action")]
    action: String,
    #[serde(rename = "errnoRet", default)]
    errno_ret: i32,
    #[serde(rename = "args", default)]
    args: Vec<CfgSyscallArg>,
    #[serde(rename = "includes", default)]
    includes: CfgFilter,
    #[serde(rename = "excludes", default)]
    excludes: CfgFilter,
}

#[derive(Deserialize)]
struct CfgSyscallArg {
    #[serde(rename = "index")]
    index: u32,
    #[serde(rename = "value")]
    value: u64,
    #[serde(rename = "valueTwo", default)]
    value_two: u64,
    #[serde(rename = "op")]
    op: String,
}

#[derive(Deserialize, Default)]
struct CfgFilter {
    #[serde(rename = "arches", default)]
    arches: Vec<String>,
    #[serde(rename = "caps", default)]
    caps: Vec<String>,
}

pub(crate) fn load(seccomp: &str) -> Result<Filter> {
    let path = format!("seccomp/{}.json", seccomp);
    let data = Assets::get(&path).expect("EmbeddedFile missing").data;
    let data = std::str::from_utf8(&data).expect("EmbeddedFile invalid");
    load_str(data)
}

// [podman#setupSeccomp]: https://github.com/containers/podman/blob/27f42775ce9bbe2957a89a02b2e48e26e0645552/vendor/github.com/containers/common/pkg/seccomp/seccomp_linux.go#L101
pub(crate) fn load_str(data: &str) -> Result<Filter> {
    let profile: CfgSeccomp = serde_json::from_str(data)?;

    let default_action = profile.default_action;
    let default_errno_ret = profile.default_errno_ret;
    let filter_default_action = translate_action(&default_action, default_errno_ret)?;
    let mut filter = Filter::new(filter_default_action);

    let runtime_arch = runtime_arch();
    for map in profile.arch_map {
        if runtime_arch == translate_arch(&map.arch)? {
            filter.add_arch(runtime_arch);
            for sub_arch in map.sub_arches {
                filter.add_arch(translate_arch(&sub_arch)?);
            }
            break;
        }
    }

    for syscall in profile.syscalls {
        let arches = &syscall.excludes.arches;
        let caps = &syscall.excludes.caps;
        if !arches.is_empty() && contains_arch(arches, runtime_arch) {
            continue;
        }
        if !caps.is_empty() && contains_caps(caps) {
            continue;
        }

        let arches = &syscall.includes.arches;
        let caps = &syscall.includes.caps;
        if !arches.is_empty() && !contains_arch(arches, runtime_arch) {
            continue;
        }
        if !caps.is_empty() && !contains_caps(caps) {
            continue;
        }

        let action = translate_action(&syscall.action, syscall.errno_ret)?;
        let args = translate_argcmps(&syscall.args)?;
        for name in syscall.names {
            filter.add_rule_conditional(action, &name, &args);
        }
    }

    Ok(filter)
}

fn translate_action(action: &str, errno: i32) -> Result<Action> {
    Ok(match action {
        "SCMP_ACT_ALLOW" => Action::Allow,
        "SCMP_ACT_ERRNO" => Action::Errno(errno),
        "SCMP_ACT_KILL_PROCESS" => Action::KillProcess,
        "SCMP_ACT_KILL_THREAD" => Action::KillThread,
        "SCMP_ACT_KILL" => Action::KillThread,
        "SCMP_ACT_LOG" => Action::Log,
        "SCMP_ACT_NOTIFY" => Action::Notify,
        "SCMP_ACT_TRACE" => Action::Trace(errno as u16),
        "SCMP_ACT_TRAP" => Action::Trap,
        _ => Err(anyhow!(format!("unknown action {:?}", action)))?,
    })
}

fn translate_arch(arch: &str) -> Result<Arch> {
    Ok(match arch {
        "SCMP_ARCH_X86" => Arch::X86,
        "SCMP_ARCH_X86_64" => Arch::X8664,
        "SCMP_ARCH_X32" => Arch::X32,
        "SCMP_ARCH_ARM" => Arch::Arm,
        "SCMP_ARCH_AARCH64" => Arch::Aarch64,
        "SCMP_ARCH_MIPS" => Arch::Mips,
        "SCMP_ARCH_MIPS64" => Arch::Mips64,
        "SCMP_ARCH_MIPS64N32" => Arch::Mips64n32,
        "SCMP_ARCH_MIPSEL" => Arch::Mipsel,
        "SCMP_ARCH_MIPSEL64" => Arch::Mipsel64,
        "SCMP_ARCH_MIPSEL64N32" => Arch::Mipsel64n32,
        "SCMP_ARCH_PPC" => Arch::Ppc,
        "SCMP_ARCH_PPC64" => Arch::Ppc64,
        "SCMP_ARCH_PPC64LE" => Arch::Ppc64le,
        "SCMP_ARCH_S390" => Arch::S390,
        "SCMP_ARCH_S390X" => Arch::S390x,
        "SCMP_ARCH_RISCV64" => Arch::Riscv64,
        _ => Err(anyhow!(format!("unknown arch {:?}", arch)))?,
    })
}

fn translate_argcmps(args: &[CfgSyscallArg]) -> Result<Vec<ArgCmp>> {
    let mut argcmps: Vec<ArgCmp> = vec![];
    for arg in args {
        let pos = arg.index;
        let op = translate_argcmp_op(&arg.op)?;
        let datum_a = arg.value;
        let datum_b = arg.value_two;
        argcmps.push(ArgCmp::new(pos, op, datum_a, datum_b));
    }
    Ok(argcmps)
}

fn translate_argcmp_op(op: &str) -> Result<ArgCmpOp> {
    Ok(match op {
        "SCMP_CMP_NE" => ArgCmpOp::Ne,
        "SCMP_CMP_LT" => ArgCmpOp::Lt,
        "SCMP_CMP_LE" => ArgCmpOp::Le,
        "SCMP_CMP_EQ" => ArgCmpOp::Eq,
        "SCMP_CMP_GE" => ArgCmpOp::Ge,
        "SCMP_CMP_GT" => ArgCmpOp::Gt,
        "SCMP_CMP_MASKED_EQ" => ArgCmpOp::MaskedEq,
        _ => Err(anyhow!(format!("unknown argcmp op {:?}", op)))?,
    })
}

fn contains_arch(arches: &[String], runtime_arch: Arch) -> bool {
    arches.iter().any(|str| match Arch::from_str(str) {
        Ok(arch) => arch == runtime_arch,
        Err(_) => false,
    })
}

fn contains_caps(_caps: &[String]) -> bool {
    // always TRUE, since we donot restrict capabilities
    true
}

fn runtime_arch() -> Arch {
    match std::env::consts::ARCH {
        "x86_64" => Arch::X8664,
        "x86" => Arch::X86,
        "arm" => Arch::Arm,
        "aarch64" => Arch::Aarch64,
        "mips" => Arch::Mips,
        "mips64" => Arch::Mips64,
        "powerpc" => Arch::Ppc,
        "powerpc64" => Arch::Ppc64,
        "s390x" => Arch::S390x,
        "riscv64" => Arch::Riscv64,
        _ => Arch::Native,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn contains_rule(rules: &[Rule], action: Action, sysname: &str) -> bool {
        rules
            .iter()
            .any(|r| r.action == action && r.sysname == sysname)
    }

    #[test]
    fn test_load() {
        let filter = load("podman").unwrap();
        let rules = filter.get_rules();

        assert!(contains_rule(&rules, Action::Allow, "accept"));
        assert!(contains_rule(&rules, Action::Allow, "brk"));
        assert!(contains_rule(&rules, Action::Allow, "read"));
        assert!(contains_rule(&rules, Action::Errno(1), "vm86"));
        assert!(contains_rule(&rules, Action::Errno(1), "vm86old"));

        #[cfg(target_arch = "x86_64")]
        {
            // includes#arches
            assert!(contains_rule(&rules, Action::Allow, "arch_prctl"));
            assert!(contains_rule(&rules, Action::Allow, "modify_ldt"));

            // includes#arches
            assert!(!contains_rule(&rules, Action::Allow, "s390_pci_mmio_read"));
            assert!(!contains_rule(&rules, Action::Allow, "s390_pci_mmio_write"));
        }

        // includes#caps
        assert!(contains_rule(&rules, Action::Allow, "sethostname"));
        assert!(contains_rule(&rules, Action::Allow, "clock_settime"));

        // excludes#caps
        assert!(!contains_rule(&rules, Action::Errno(1), "sethostname"));
        assert!(!contains_rule(&rules, Action::Errno(1), "clock_settime"));
    }
}
