use nix::{
    sys::signal::{SaFlags, SigAction, SigHandler, SigSet, Signal},
    unistd::Pid,
};

use crate::child_process::{error::Result, syscall};

static mut GRANDCHILD: libc::pid_t = 0;

extern "C" fn signal_handler(_: libc::c_int) {
    unsafe {
        if GRANDCHILD == 0 {
            return;
        }

        libc::kill(GRANDCHILD, libc::SIGKILL);
    }
}

pub(crate) fn init(timeout: u64, grandchild: Pid) -> Result<()> {
    unsafe {
        GRANDCHILD = grandchild.as_raw();
    }

    let sa = SigAction::new(
        SigHandler::Handler(signal_handler),
        SaFlags::SA_RESTART,
        SigSet::empty(),
    );
    syscall::sigaction(Signal::SIGALRM, &sa)?;
    syscall::setalarm(timeout)
}
