use nix::sched;
use nix::sys::signal;
use nix::sys::{prctl, resource, wait};
use nix::unistd::{self, alarm};
use std::ffi::CStr;
use std::fmt::Debug;
use std::fs;
use std::io;
use std::os::unix::io::RawFd;
use std::path::Path;

pub(crate) use nix::sched::CloneFlags;
pub(crate) use nix::sys::resource::{Resource, Usage, UsageWho};
pub(crate) use nix::sys::signal::{SaFlags, SigAction, SigHandler, SigSet, Signal};
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

pub(crate) fn dup2(oldfd: RawFd, newfd: RawFd) -> Result<RawFd> {
    map_err!(unistd::dup2(oldfd, newfd))
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

pub(crate) fn fwrite<P: AsRef<Path> + Debug>(path: P, content: &str) -> Result<()> {
    fs::write(path.as_ref(), content.as_bytes()).map_err(|err| {
        let err = format!("write({:?}, ...) => {}", path.as_ref(), err);
        Error::NixError(err)
    })
}

pub(crate) fn getrusage(who: UsageWho) -> Result<Usage> {
    map_err!(resource::getrusage(who))
}

pub(crate) fn set_pdeathsig(sig: Signal) -> Result<()> {
    map_err!(prctl::set_pdeathsig(sig))
}

pub(crate) fn sigaction(signal: Signal, sigaction: &SigAction) -> Result<SigAction> {
    unsafe { signal::sigaction(signal, sigaction) }.map_err(|err| {
        let err = format!("sigaction({:?}, ...) => {}", signal, err);
        Error::NixError(err)
    })
}

pub(crate) fn setalarm(secs: u64) -> Result<()> {
    alarm::set(secs as u32);
    Ok(())
}

pub(crate) fn sethostname(hostname: &str) -> Result<()> {
    map_err!(unistd::sethostname(hostname))
}

pub(crate) fn setrlimit(resource: Resource, soft_limit: u64, hard_limit: u64) -> Result<()> {
    map_err!(resource::setrlimit(resource, soft_limit, hard_limit))
}

pub(crate) fn setsid() -> Result<Pid> {
    map_err!(unistd::setsid())
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
