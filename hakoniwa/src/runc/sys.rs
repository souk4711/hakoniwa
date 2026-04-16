use nix::errno::Errno;
use nix::mount;
use nix::sched;
use nix::sys::{prctl, ptrace, resource, signal, statfs, wait};
use nix::unistd::{self, alarm};
use std::ffi::{CStr, CString, OsStr};
use std::fmt::Debug;
use std::fs;
use std::fs::{Metadata, OpenOptions};
use std::os::fd::RawFd;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::{self as unix_fs, PermissionsExt};

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

const ERRNO_SENTINEL: i32 = -1;
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
            Error::SysError(format!("{}({}) => {}", name, args, err))
        })
    };
}

pub(crate) fn unshare(clone_flags: CloneFlags) -> Result<()> {
    map_err!(sched::unshare(clone_flags))
}

pub(crate) fn fork() -> Result<ForkResult> {
    unsafe { unistd::fork() }.map_err(|err| {
        let err = format!("fork() => {err}");
        Error::SysError(err)
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
        Error::SysError(err)
    })
}

pub(crate) fn sigraise(sig: Signal) -> Result<()> {
    map_err!(signal::raise(sig))
}

pub(crate) fn reset_sigpipe() -> Result<SigHandler> {
    unsafe { signal::signal(signal::SIGPIPE, SigHandler::SigDfl) }.map_err(|err| {
        let err = format!("signal(SIGPIPE, SIG_DFL) => {err}");
        Error::SysError(err)
    })
}

pub(crate) fn setalarm(secs: u64) -> Result<()> {
    alarm::set(secs as u32);
    Ok(())
}

pub(crate) fn close_extra_fds_exclude(reader: RawFd, writer: RawFd) -> Result<()> {
    let mut keep_fds = [reader, writer];
    keep_fds.sort_unstable();

    unsafe {
        close_fds::close_open_fds(3, &keep_fds);
    }
    Ok(())
}

pub(crate) fn close_stdin() -> Result<()> {
    unistd::close(libc::STDIN_FILENO).map_err(|err| {
        let err = format!("close_stdin() => {err}");
        Error::SysError(err)
    })
}

pub(crate) fn close_stdout() -> Result<()> {
    unistd::close(libc::STDOUT_FILENO).map_err(|err| {
        let err = format!("close_stdout() => {err}");
        Error::SysError(err)
    })
}

pub(crate) fn close_stderr() -> Result<()> {
    unistd::close(libc::STDERR_FILENO).map_err(|err| {
        let err = format!("close_stderr() => {err}");
        Error::SysError(err)
    })
}

pub(crate) fn dup2_stdin(oldfd: RawFd) -> Result<RawFd> {
    unistd::dup2(oldfd, libc::STDIN_FILENO).map_err(|err| {
        let err = format!("dup2_stdin(..) => {err}");
        Error::SysError(err)
    })
}

pub(crate) fn dup2_stdout(oldfd: RawFd) -> Result<RawFd> {
    unistd::dup2(oldfd, libc::STDOUT_FILENO).map_err(|err| {
        let err = format!("dup2_stdout(..) => {err}");
        Error::SysError(err)
    })
}

pub(crate) fn dup2_stderr(oldfd: RawFd) -> Result<RawFd> {
    unistd::dup2(oldfd, libc::STDERR_FILENO).map_err(|err| {
        let err = format!("dup2_stderr(..) => {err}");
        Error::SysError(err)
    })
}

pub(crate) fn write_stderr(buf: &[u8]) -> Result<usize> {
    nix_unistd_write(libc::STDERR_FILENO, buf).map_err(|err| {
        let err = format!("write(STDERR, ..) => {err}");
        Error::SysError(err)
    })
}

pub(crate) fn fwrite<P: AsRef<Path> + Debug>(path: P, content: &str) -> Result<()> {
    fs::write(path.as_ref(), content.as_bytes()).map_err(|err| {
        let err = format!("write({path:?}, ..) => {err}");
        Error::SysError(err)
    })
}

pub(crate) fn touch<P: AsRef<Path> + Debug>(path: P) -> Result<()> {
    OpenOptions::new()
        .create(true)
        .append(true)
        .open(path.as_ref())
        .map(|_| ())
        .map_err(|err| {
            let err = format!("touch({path:?}) => {err}");
            Error::SysError(err)
        })
}

pub(crate) fn symlink<P1: AsRef<Path> + Debug, P2: AsRef<Path> + Debug>(
    original: P1,
    link: P2,
) -> Result<()> {
    if let Ok(path) = fs::read_link(link.as_ref())
        && path == original.as_ref()
    {
        return Ok(());
    }

    unix_fs::symlink(original.as_ref(), link.as_ref()).map_err(|err| {
        let err = format!("symlink({original:?}, {link:?}) => {err}");
        Error::SysError(err)
    })
}

pub(crate) fn mkdir_p<P: AsRef<Path> + Debug>(path: P) -> Result<()> {
    fs::create_dir_all(path.as_ref()).map_err(|err| {
        let err = format!("mkdir_p({path:?}) => {err}");
        Error::SysError(err)
    })
}

pub(crate) fn rmdir<P: AsRef<Path> + Debug>(path: P) -> Result<()> {
    fs::remove_dir(path.as_ref()).map_err(|err| {
        let err = format!("rmdir({path:?}) => {err}");
        Error::SysError(err)
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

pub(crate) fn mount_filesystem_with_data<P1: AsRef<Path> + Debug, P2: AsRef<Path> + Debug>(
    fstype: &str,
    source: P1,
    target: P2,
    flags: MsFlags,
    data: &str,
) -> Result<()> {
    let source = source.as_ref();
    let target = target.as_ref();
    map_err!(mount::mount(
        Some(source),
        target,
        Some(fstype),
        flags,
        Some(data)
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

pub(crate) fn sethostname(hostname: &str) -> Result<()> {
    map_err!(unistd::sethostname(hostname))
}

pub(crate) fn setuid(uid: u32) -> Result<()> {
    // [Use raw syscalls to avoid sporadic hangs]: https://github.com/youki-dev/youki/pull/2425
    if unsafe { libc::syscall(libc::SYS_setresuid, uid, uid, uid) } == ERRNO_SENTINEL as i64 {
        let err = Errno::last();
        let err = format!("setuid({uid}) => {err}");
        Err(Error::SysError(err))
    } else {
        Ok(())
    }
}

pub(crate) fn setgid(gid: u32) -> Result<()> {
    // [Use raw syscalls to avoid sporadic hangs]: https://github.com/youki-dev/youki/pull/2425
    if unsafe { libc::syscall(libc::SYS_setresgid, gid, gid, gid) } == ERRNO_SENTINEL as i64 {
        let err = Errno::last();
        let err = format!("setgid({gid}) => {err}");
        Err(Error::SysError(err))
    } else {
        Ok(())
    }
}

pub(crate) fn setgroups(groups: &[u32]) -> Result<()> {
    // [Use raw syscalls to avoid sporadic hangs]: https://github.com/youki-dev/youki/pull/2425
    let ngroups = groups.len() as libc::size_t;
    let ptr = groups.as_ptr() as *const libc::gid_t;
    if unsafe { libc::syscall(libc::SYS_setgroups, ngroups, ptr) } == ERRNO_SENTINEL as i64 {
        let err = Errno::last();
        let err = format!("setgroups(..) => {err}");
        Err(Error::SysError(err))
    } else {
        Ok(())
    }
}

pub(crate) fn setenv(k: &str, v: &str) -> Result<()> {
    let key = CString::new(k)?;
    let value = CString::new(v)?;
    if unsafe { libc::setenv(key.as_ptr(), value.as_ptr(), 1) } == ERRNO_SENTINEL {
        let err = Errno::last();
        let err = format!("setenv({k}, {v}) => {err}");
        Err(Error::SysError(err))
    } else {
        Ok(())
    }
}

pub(crate) fn clearenv() -> Result<()> {
    if unsafe { libc::clearenv() } != 0 {
        let err = "clearenv failed";
        let err = format!("clearenv() => {err}");
        Err(Error::SysError(err))
    } else {
        Ok(())
    }
}

pub(crate) fn isatty() -> Result<bool> {
    nix_unistd_isatty(libc::STDOUT_FILENO).map_err(|err| {
        let err = format!("isatty(STDOUT) => {err}");
        Error::SysError(err)
    })
}

pub(crate) fn ttyname() -> Result<PathBuf> {
    nix_unistd_ttyname(libc::STDOUT_FILENO).map_err(|err| {
        let err = format!("ttyname(STDOUT) => {err}");
        Error::SysError(err)
    })
}

// [nix::unistd::write]: https://github.com/nix-rust/nix/blob/v0.31.2/src/unistd.rs#L1379
fn nix_unistd_write(fd: RawFd, buf: &[u8]) -> nix::Result<usize> {
    let res = unsafe { libc::write(fd, buf.as_ptr().cast(), buf.len() as libc::size_t) };
    Errno::result(res).map(|r| r as usize)
}

// [nix::unistd::isatty]: https://github.com/nix-rust/nix/blob/v0.31.2/src/unistd.rs#L1543
fn nix_unistd_isatty(fd: RawFd) -> nix::Result<bool> {
    unsafe {
        // ENOTTY means `fd` is a valid file descriptor, but not a TTY, so
        // we return `Ok(false)`
        if libc::isatty(fd) == 1 {
            Ok(true)
        } else {
            match Errno::last() {
                Errno::ENOTTY => Ok(false),
                err => Err(err),
            }
        }
    }
}

// [nix::unistd::ttyname]: https://github.com/nix-rust/nix/blob/v0.31.2/src/unistd.rs#L3972
fn nix_unistd_ttyname(fd: RawFd) -> nix::Result<PathBuf> {
    const PATH_MAX: usize = libc::PATH_MAX as usize;
    let mut buf = vec![0_u8; PATH_MAX];
    let c_buf = buf.as_mut_ptr().cast();

    let ret = unsafe { libc::ttyname_r(fd, c_buf, buf.len()) };
    if ret != 0 {
        return Err(Errno::from_raw(ret));
    }

    CStr::from_bytes_until_nul(&buf[..])
        .map(|s| OsStr::from_bytes(s.to_bytes()).into())
        .map_err(|_| Errno::EINVAL)
}
