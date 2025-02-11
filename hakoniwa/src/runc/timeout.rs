use crate::runc::error::*;
use crate::runc::nix::{self, Pid, SaFlags, SigAction, SigHandler, SigSet, Signal};

static mut CHILD: libc::pid_t = 0;

extern "C" fn signal_handler(_: libc::c_int) {
    unsafe {
        if CHILD != 0 {
            libc::kill(CHILD, libc::SIGKILL);
        }
    }
}

pub(crate) fn timeout(child: Pid, timeout: u64) -> Result<()> {
    unsafe {
        CHILD = child.as_raw();
    }

    let sa = SigAction::new(
        SigHandler::Handler(signal_handler),
        SaFlags::SA_RESTART,
        SigSet::empty(),
    );
    nix::sigaction(Signal::SIGALRM, &sa)?;
    nix::setalarm(timeout)
}
