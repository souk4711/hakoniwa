use nix::sched;
use nix::sys::resource;
use nix::sys::wait;
use nix::unistd;
use std::ffi::CStr;
use std::fmt::Debug;
use std::io;

pub(crate) use nix::sched::CloneFlags;
pub(crate) use nix::sys::resource::{Resource, Usage, UsageWho};
pub(crate) use nix::sys::wait::{WaitPidFlag, WaitStatus};
pub(crate) use nix::unistd::{ForkResult, Pid};

use crate::runc::error::*;

macro_rules! map_err {
    ($mod:ident :: $fn:ident ()) => {
        map_err!($mod::$fn(), "")
    };

    ($mod:ident :: $fn:ident ($arg1:expr)) => {
        map_err!($mod::$fn($arg1), "{:?}")
    };

    ($mod:ident :: $fn:ident ($arg1:expr, $arg2:expr)) => {
        map_err!($mod::$fn($arg1, $arg2), "{:?}, {:?}")
    };

    ($mod:ident :: $fn:ident ($arg1:expr, $arg2:expr, $arg3:expr)) => {
        map_err!($mod::$fn($arg1, $arg2, $arg3), "{:?}, {:?}, {:?}")
    };

    ($mod:ident :: $fn:ident ($arg1:expr, $arg2:expr, $arg3:expr, $arg4:expr)) => {
        map_err!($mod::$fn($arg1, $arg2, $arg3, $arg4), "{:?}, {:?}, {:?}, {:?}")
    };

    ($mod:ident :: $fn:ident ($arg1:expr, $arg2:expr, $arg3:expr, $arg4:expr, $arg5:expr)) => {
        map_err!($mod::$fn($arg1, $arg2, $arg3, $arg4, $arg5), "{:?}, {:?}, {:?}, {:?}, {:?}")
    };

    ($mod:ident :: $fn:ident ($($arg:expr),* ), $args_format:literal) => {
        $mod::$fn($($arg),*).map_err(|err| {
            let name = stringify!($fn);
            let args = format!($args_format, $($arg),*);
            Error::NixError(format!("{}({}) => {}", name, args, err))
        })
    };
}

pub(crate) fn execve<S1: AsRef<CStr> + Debug, S2: AsRef<CStr> + Debug>(
    prog: &CStr,
    argv: &[S1],
    envp: &[S2],
) -> Result<()> {
    map_err!(unistd::execve(prog, argv, envp))?;
    Ok(())
}

pub(crate) fn fork() -> Result<ForkResult> {
    unsafe { unistd::fork() }.map_err(|err| {
        let err = format!("fork() => {}", err);
        Error::NixError(err)
    })
}

pub(crate) fn getrusage(who: UsageWho) -> Result<Usage> {
    map_err!(resource::getrusage(who))
}

pub(crate) fn prctl_set_pdeathsig(sig: i32) -> Result<()> {
    let res = unsafe { libc::prctl(libc::PR_SET_PDEATHSIG, sig, 0, 0, 0) };
    if res == -1 {
        let err = format!("prctl(PR_SET_PDEATHSIG, {:?}, ...) => {}", sig, res);
        Err(Error::NixError(err))
    } else {
        Ok(())
    }
}

pub(crate) fn setsid() -> Result<Pid> {
    map_err!(unistd::setsid())
}

pub(crate) fn setrlimit(resource: Resource, soft_limit: u64, hard_limit: u64) -> Result<()> {
    map_err!(resource::setrlimit(resource, soft_limit, hard_limit))
}

pub(crate) fn unshare(clone_flags: CloneFlags) -> Result<()> {
    map_err!(sched::unshare(clone_flags))
}

pub(crate) fn waitpid(pid: Pid) -> Result<WaitStatus> {
    map_err!(wait::waitpid(pid, None::<WaitPidFlag>))
}

pub(crate) fn write_stderr(buf: &[u8]) -> Result<usize> {
    unistd::write(io::stderr(), buf).map_err(|err| {
        let err = format!("write(STDERR, ...) => {}", err);
        Error::NixError(err)
    })
}
