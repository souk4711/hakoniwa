use nix::{
    mount::{self, MntFlags, MsFlags},
    sched::{self, CloneFlags},
    sys::resource::{self, Resource, Usage, UsageWho},
    sys::signal::{self, SigAction, Signal},
    sys::wait::{self, WaitPidFlag, WaitStatus},
    unistd::{self, alarm, ForkResult, Pid},
};
use std::{
    ffi::CStr,
    fmt::Debug,
    fs::{self, File, Metadata},
    os::unix::io::RawFd,
    path::Path,
};

use crate::child_process::error::{Error, Result};

const NULL: Option<&'static Path> = None;

macro_rules! tryfn {
    ($mod:ident :: $fn:ident ()) => {
        tryfn!($mod::$fn(), "")
    };

    ($mod:ident :: $fn:ident ($arg1:expr)) => {
        tryfn!($mod::$fn($arg1), "{:?}")
    };

    ($mod:ident :: $fn:ident ($arg1:expr, $arg2:expr)) => {
        tryfn!($mod::$fn($arg1, $arg2), "{:?}, {:?}")
    };

    ($mod:ident :: $fn:ident ($arg1:expr, $arg2:expr, $arg3:expr)) => {
        tryfn!($mod::$fn($arg1, $arg2, $arg3), "{:?}, {:?}, {:?}")
    };

    ($mod:ident :: $fn:ident ($arg1:expr, $arg2:expr, $arg3:expr, $arg4:expr)) => {
        tryfn!($mod::$fn($arg1, $arg2, $arg3, $arg4), "{:?}, {:?}, {:?}, {:?}")
    };

    ($mod:ident :: $fn:ident ($arg1:expr, $arg2:expr, $arg3:expr, $arg4:expr, $arg5:expr)) => {
        tryfn!($mod::$fn($arg1, $arg2, $arg3, $arg4, $arg5), "{:?}, {:?}, {:?}, {:?}, {:?}")
    };

    ($mod:ident :: $fn:ident ($($arg:expr),* ), $args_format:literal) => {
        $mod::$fn($($arg),*).map_err(|err| {
            let name = stringify!($fn);
            let args = format!($args_format, $($arg),*);
            Error(format!("{}({}) => {}", name, args, err))
        })
    };
}

pub fn metadata<P: AsRef<Path> + Debug>(path: P) -> Result<Metadata> {
    tryfn!(fs::metadata(path.as_ref()))
}

pub fn mkdir_p<P: AsRef<Path> + Debug>(path: P) -> Result<()> {
    fs::create_dir_all(path.as_ref()).map_err(|err| {
        let err = format!("mkdir_p({:?}) => {}", path.as_ref(), err);
        Error(err)
    })
}

pub fn rmdir<P: AsRef<Path> + Debug>(path: P) -> Result<()> {
    fs::remove_dir(path.as_ref()).map_err(|err| {
        let err = format!("rmdir({:?}) => {}", path.as_ref(), err);
        Error(err)
    })
}

pub fn chdir<P: AsRef<Path> + Debug>(path: P) -> Result<()> {
    tryfn!(unistd::chdir(path.as_ref()))
}

pub fn touch<P: AsRef<Path> + Debug>(path: P) -> Result<()> {
    File::create(path.as_ref()).map(|_| ()).map_err(|err| {
        let err = format!("touch({:?}) => {}", path.as_ref(), err);
        Error(err)
    })
}

pub fn read(fd: RawFd, buf: &mut [u8]) -> Result<usize> {
    unistd::read(fd, buf).map_err(|err| {
        let err = format!("read({:?}, ...) => {}", fd, err);
        Error(err)
    })
}

pub fn write(fd: RawFd, buf: &[u8]) -> Result<usize> {
    unistd::write(fd, buf).map_err(|err| {
        let err = format!("write({:?}, ...) => {}", fd, err);
        Error(err)
    })
}

pub fn close(fd: RawFd) -> Result<()> {
    tryfn!(unistd::close(fd))
}

pub fn dup2(oldfd: RawFd, newfd: RawFd) -> Result<RawFd> {
    tryfn!(unistd::dup2(oldfd, newfd))
}

pub fn fwrite<P: AsRef<Path> + Debug>(path: P, content: &str) -> Result<()> {
    fs::write(path.as_ref(), content.as_bytes()).map_err(|err| {
        let err = format!("write({:?}, ...) => {}", path.as_ref(), err);
        Error(err)
    })
}

pub fn fsync(fd: RawFd) -> Result<()> {
    tryfn!(unistd::fsync(fd))
}

pub fn mount<P1: AsRef<Path> + Debug, P2: AsRef<Path> + Debug>(
    source: P1,
    target: P2,
    flags: MsFlags,
) -> Result<()> {
    let (source, target) = (source.as_ref(), target.as_ref());
    tryfn!(mount::mount(Some(source), target, NULL, flags, NULL))
}

pub fn mount_root() -> Result<()> {
    let flags = MsFlags::MS_PRIVATE | MsFlags::MS_REC;
    tryfn!(mount::mount(NULL, "/", NULL, flags, NULL))
}

pub fn mount_proc<P: AsRef<Path> + Debug>(target: P) -> Result<()> {
    let target = target.as_ref();
    let flags = MsFlags::MS_NOSUID | MsFlags::MS_NODEV | MsFlags::MS_NOEXEC;
    tryfn!(mount::mount(NULL, target, Some("proc"), flags, NULL))
}

pub fn mount_tmpfs<P: AsRef<Path> + Debug>(target: P) -> Result<()> {
    let target = target.as_ref();
    let flags = MsFlags::MS_NOSUID | MsFlags::MS_NODEV | MsFlags::MS_NOEXEC;
    tryfn!(mount::mount(NULL, target, Some("tmpfs"), flags, NULL))
}

pub fn unmount<P: AsRef<Path> + Debug>(target: P) -> Result<()> {
    let flags = MntFlags::MNT_DETACH;
    tryfn!(mount::umount2(target.as_ref(), flags))
}

pub fn pivot_root<P1: AsRef<Path> + Debug, P2: AsRef<Path> + Debug>(
    new_root: P1,
    put_old: P2,
) -> Result<()> {
    tryfn!(unistd::pivot_root(new_root.as_ref(), put_old.as_ref()))
}

pub fn unshare(clone_flags: CloneFlags) -> Result<()> {
    tryfn!(sched::unshare(clone_flags))
}

pub fn fork() -> Result<ForkResult> {
    unsafe { unistd::fork() }.map_err(|err| {
        let err = format!("fork() => {}", err);
        Error(err)
    })
}

pub fn execve<SA: AsRef<CStr> + Debug, SE: AsRef<CStr> + Debug>(
    prog: &CStr,
    argv: &[SA],
    env: &[SE],
) -> Result<()> {
    tryfn!(unistd::execve(prog, argv, env))?;
    Ok(())
}

pub fn waitpid(pid: Pid) -> Result<WaitStatus> {
    tryfn!(wait::waitpid(pid, None::<WaitPidFlag>))
}

pub fn getrusage(who: UsageWho) -> Result<Usage> {
    tryfn!(resource::getrusage(who))
}

pub fn setsid() -> Result<Pid> {
    tryfn!(unistd::setsid())
}

pub fn setrlimit(resource: Resource, limit: Option<u64>) -> Result<()> {
    match limit {
        Some(limit) => tryfn!(resource::setrlimit(resource, limit, limit)),
        None => Ok(()),
    }
}

pub fn prctl_set_pdeathsig(sig: i32) -> Result<()> {
    let res = unsafe { libc::prctl(libc::PR_SET_PDEATHSIG, sig, 0, 0, 0) };
    if res == -1 {
        let err = format!("prctl(PR_SET_PDEATHSIG, {:?}, ...) => {}", sig, res);
        Err(Error(err))
    } else {
        Ok(())
    }
}

pub fn sigaction(signal: Signal, sigaction: &SigAction) -> Result<SigAction> {
    unsafe { signal::sigaction(signal, sigaction) }.map_err(|err| {
        let err = format!("sigaction({:?}, ...) => {}", signal, err);
        Error(err)
    })
}

pub fn setalarm(secs: u64) -> Result<()> {
    alarm::set(secs as u32);
    Ok(())
}

pub fn sethostname(hostname: &str) -> Result<()> {
    tryfn!(unistd::sethostname(hostname))
}
