use nix::mount;
use nix::sched;
use nix::sys::{prctl, ptrace, resource, signal, statfs, wait};
use nix::unistd::{self, alarm};
use std::ffi::CStr;
use std::fmt::Debug;
use std::fs;
use std::fs::{File, Metadata};
use std::io;
use std::os::unix::fs as unix_fs;
use std::os::unix::fs::PermissionsExt;

pub(crate) use nix::mount::{MntFlags, MsFlags};
pub(crate) use nix::sched::CloneFlags;
pub(crate) use nix::sys::ptrace::Event as PtraceEvent;
pub(crate) use nix::sys::resource::{Resource, Usage, UsageWho};
pub(crate) use nix::sys::signal::{SaFlags, SigAction, SigHandler, SigSet, Signal};
pub(crate) use nix::sys::statfs::Statfs;
pub(crate) use nix::sys::statvfs::FsFlags;
pub(crate) use nix::sys::wait::{WaitPidFlag, WaitStatus};
pub(crate) use nix::unistd::{ForkResult, Pid};
pub(crate) use std::path::{Path, PathBuf};

use super::error::*;

const NULL: Option<&'static Path> = None;

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

pub(crate) fn unshare(clone_flags: CloneFlags) -> Result<()> {
    map_err!(sched::unshare(clone_flags))
}

pub(crate) fn fork() -> Result<ForkResult> {
    unsafe { unistd::fork() }.map_err(|err| {
        let err = format!("fork() => {err}");
        Error::NixError(err)
    })
}

pub(crate) fn execve<S1: AsRef<CStr> + Debug, S2: AsRef<CStr> + Debug>(
    prog: &CStr,
    argv: &[S1],
    envp: &[S2],
) -> Result<()> {
    map_err!(unistd::execve(prog, argv, envp))?;
    Ok(())
}

pub(crate) fn waitpid(pid: Pid) -> Result<WaitStatus> {
    map_err!(wait::waitpid(pid, None::<WaitPidFlag>))
}

pub(crate) fn ptrace_traceexit(pid: Pid) -> Result<()> {
    map_err!(ptrace::setoptions(pid, ptrace::Options::PTRACE_O_TRACEEXIT))
}

pub(crate) fn ptrace_cont(pid: Pid, signal: Option<Signal>) -> Result<()> {
    map_err!(ptrace::cont(pid, signal))
}

pub(crate) fn traceme() -> Result<()> {
    map_err!(ptrace::traceme())
}

pub(crate) fn getrusage(who: UsageWho) -> Result<Usage> {
    map_err!(resource::getrusage(who))
}

pub(crate) fn setrlimit(resource: Resource, soft_limit: u64, hard_limit: u64) -> Result<()> {
    map_err!(resource::setrlimit(resource, soft_limit, hard_limit))
}

pub(crate) fn set_pdeathsig(sig: Signal) -> Result<()> {
    map_err!(prctl::set_pdeathsig(sig))
}

pub(crate) fn set_no_new_privs() -> Result<()> {
    map_err!(prctl::set_no_new_privs())
}

pub(crate) fn set_keepcaps(attribute: bool) -> Result<()> {
    map_err!(prctl::set_keepcaps(attribute))
}

pub(crate) fn sigaction(signal: Signal, sigaction: &SigAction) -> Result<SigAction> {
    unsafe { signal::sigaction(signal, sigaction) }.map_err(|err| {
        let err = format!("sigaction({signal:?}, ..) => {err}");
        Error::NixError(err)
    })
}

pub(crate) fn sigraise(sig: Signal) -> Result<()> {
    map_err!(signal::raise(sig))
}

pub(crate) fn reset_sigpipe() -> Result<SigHandler> {
    unsafe { signal::signal(signal::SIGPIPE, SigHandler::SigDfl) }.map_err(|err| {
        let err = format!("signal(SIGPIPE, SIG_DFL) => {err}");
        Error::NixError(err)
    })
}

pub(crate) fn setalarm(secs: u64) -> Result<()> {
    alarm::set(secs as u32);
    Ok(())
}

pub(crate) fn dup2_stdin<Fd: std::os::fd::AsFd>(oldfd: Fd) -> Result<()> {
    unistd::dup2_stdin(oldfd).map_err(|err| {
        let err = format!("dup2_stdin(..) => {err}");
        Error::NixError(err)
    })
}

pub(crate) fn dup2_stdout<Fd: std::os::fd::AsFd>(oldfd: Fd) -> Result<()> {
    unistd::dup2_stdout(oldfd).map_err(|err| {
        let err = format!("dup2_stdout(..) => {err}");
        Error::NixError(err)
    })
}

pub(crate) fn dup2_stderr<Fd: std::os::fd::AsFd>(oldfd: Fd) -> Result<()> {
    unistd::dup2_stderr(oldfd).map_err(|err| {
        let err = format!("dup2_stderr(..) => {err}");
        Error::NixError(err)
    })
}

pub(crate) fn write_stderr(buf: &[u8]) -> Result<usize> {
    unistd::write(io::stderr(), buf).map_err(|err| {
        let err = format!("write(STDERR, ..) => {err}");
        Error::NixError(err)
    })
}

pub(crate) fn fwrite<P: AsRef<Path> + Debug>(path: P, content: &str) -> Result<()> {
    fs::write(path.as_ref(), content.as_bytes()).map_err(|err| {
        let err = format!("write({path:?}, ..) => {err}");
        Error::NixError(err)
    })
}

pub(crate) fn touch<P: AsRef<Path> + Debug>(path: P) -> Result<()> {
    File::create(path.as_ref()).map(|_| ()).map_err(|err| {
        let err = format!("touch({path:?}) => {err}");
        Error::NixError(err)
    })
}

pub(crate) fn symlink<P1: AsRef<Path> + Debug, P2: AsRef<Path> + Debug>(
    original: P1,
    link: P2,
) -> Result<()> {
    unix_fs::symlink(original.as_ref(), link.as_ref()).map_err(|err| {
        let err = format!("symlink({original:?}, {link:?}) => {err}");
        Error::NixError(err)
    })
}

pub(crate) fn mkdir_p<P: AsRef<Path> + Debug>(path: P) -> Result<()> {
    fs::create_dir_all(path.as_ref()).map_err(|err| {
        let err = format!("mkdir_p({path:?}) => {err}");
        Error::NixError(err)
    })
}

pub(crate) fn rmdir<P: AsRef<Path> + Debug>(path: P) -> Result<()> {
    fs::remove_dir(path.as_ref()).map_err(|err| {
        let err = format!("rmdir({path:?}) => {err}");
        Error::NixError(err)
    })
}

pub(crate) fn chdir<P: AsRef<Path> + Debug>(path: P) -> Result<()> {
    map_err!(unistd::chdir(path.as_ref()))
}

pub(crate) fn chmod<P: AsRef<Path> + Debug>(path: P, mode: u32) -> Result<()> {
    let permissions = fs::Permissions::from_mode(mode);
    map_err!(fs::set_permissions(path.as_ref(), permissions.clone()))
}

pub(crate) fn statfs<P: AsRef<Path> + Debug>(path: P) -> Result<Statfs> {
    map_err!(statfs::statfs(path.as_ref()))
}

pub(crate) fn metadata<P: AsRef<Path> + Debug>(path: P) -> Result<Metadata> {
    map_err!(fs::metadata(path.as_ref()))
}

pub(crate) fn pivot_root<P1: AsRef<Path> + Debug, P2: AsRef<Path> + Debug>(
    new_root: P1,
    put_old: P2,
) -> Result<()> {
    map_err!(unistd::pivot_root(new_root.as_ref(), put_old.as_ref()))
}

pub(crate) fn mount<P1: AsRef<Path> + Debug, P2: AsRef<Path> + Debug>(
    source: P1,
    target: P2,
    flags: MsFlags,
) -> Result<()> {
    let (source, target) = (source.as_ref(), target.as_ref());
    map_err!(mount::mount(Some(source), target, NULL, flags, NULL))
}

pub(crate) fn mount_filesystem<P1: AsRef<Path> + Debug, P2: AsRef<Path> + Debug>(
    fstype: &str,
    source: P1,
    target: P2,
    flags: MsFlags,
) -> Result<()> {
    let source = source.as_ref();
    let target = target.as_ref();
    map_err!(mount::mount(
        Some(source),
        target,
        Some(fstype),
        flags,
        NULL
    ))
}

pub(crate) fn mount_make_private<P: AsRef<Path> + Debug>(target: P) -> Result<()> {
    let target = target.as_ref();
    let flags = MsFlags::MS_PRIVATE | MsFlags::MS_REC;
    map_err!(mount::mount(NULL, target, NULL, flags, NULL))
}

pub(crate) fn unmount<P: AsRef<Path> + Debug>(target: P) -> Result<()> {
    let flags = MntFlags::MNT_DETACH;
    map_err!(mount::umount2(target.as_ref(), flags))
}

pub(crate) fn setuid(uid: u32) -> Result<()> {
    if unsafe { libc::syscall(libc::SYS_setresuid, uid, uid, uid) } == -1 {
        let err = nix::errno::Errno::last();
        let err = format!("setuid({uid}) => {err}");
        Err(Error::NixError(err))
    } else {
        Ok(())
    }
}

pub(crate) fn setgid(gid: u32) -> Result<()> {
    if unsafe { libc::syscall(libc::SYS_setresgid, gid, gid, gid) } == -1 {
        let err = nix::errno::Errno::last();
        let err = format!("setgid({gid}) => {err}");
        Err(Error::NixError(err))
    } else {
        Ok(())
    }
}

pub(crate) fn setgroups(groups: &[u32]) -> Result<()> {
    let ngroups = groups.len() as libc::size_t;
    let ptr = groups.as_ptr() as *const libc::gid_t;
    if unsafe { libc::syscall(libc::SYS_setgroups, ngroups, ptr) } == -1 {
        let err = nix::errno::Errno::last();
        let err = format!("setgroups(..) => {err}");
        Err(Error::NixError(err))
    } else {
        Ok(())
    }
}

pub(crate) fn sethostname(hostname: &str) -> Result<()> {
    map_err!(unistd::sethostname(hostname))
}

pub(crate) fn isatty() -> Result<bool> {
    unistd::isatty(io::stdout()).map_err(|err| {
        let err = format!("isatty(STDOUT) => {err}");
        Error::NixError(err)
    })
}

pub(crate) fn ttyname() -> Result<PathBuf> {
    unistd::ttyname(io::stdout()).map_err(|err| {
        let err = format!("ttyname(STDOUT) => {err}");
        Error::NixError(err)
    })
}
