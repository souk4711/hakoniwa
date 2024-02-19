use libseccomp::{ScmpFilterContext, ScmpSyscall};

use crate::{child_process::error::Result, Seccomp};

pub(crate) fn init(seccomp: &Option<Seccomp>) -> Result<()> {
    if let Some(seccomp) = seccomp {
        let mut scmp_filter = ScmpFilterContext::new_filter(seccomp.dismatch_action())?;
        for syscall in seccomp.syscalls.iter() {
            let syscall = ScmpSyscall::from_name(syscall)?;
            scmp_filter.add_rule(seccomp.match_action(), syscall)?;
        }
        scmp_filter.load()?;
    } else {
        super::syscall::prctl_set_no_new_privs()?;
    }
    Ok(())
}
