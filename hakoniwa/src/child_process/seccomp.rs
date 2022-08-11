use libseccomp::{ScmpAction, ScmpFilterContext, ScmpSyscall};

use crate::{
    child_process::{error::map_err, error::Error, error::Result},
    Seccomp,
};

pub fn init(seccomp: &Option<Seccomp>) -> Result<()> {
    if let Some(seccomp) = seccomp {
        let mut scmp_filter = map_err!(ScmpFilterContext::new_filter(ScmpAction::KillProcess))?;
        for syscall in seccomp.syscalls.iter() {
            let syscall = map_err!(ScmpSyscall::from_name(syscall))?;
            map_err!(scmp_filter.add_rule(ScmpAction::Allow, syscall))?;
        }
        map_err!(scmp_filter.load())?;
    }
    Ok(())
}
