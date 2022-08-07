use nix::{
    mount::{self, MntFlags, MsFlags},
    sched::{self, CloneFlags},
    sys::resource::{self, Resource},
    sys::stat::{self, Mode, SFlag},
    sys::wait::{self, WaitPidFlag, WaitStatus},
    unistd::{self, ForkResult, Pid},
};
use std::{
    ffi::CStr,
    fmt::Debug,
    fs::{self, File, Metadata},
    path::Path,
};

use crate::child_process::error::{Error, Result};

const NULL: Option<&'static Path> = None;

mod alias {
    pub use std::fs::create_dir_all as mkdir;
    pub use std::fs::remove_dir as rmdir;
}

macro_rules! tryfn {
    ($fn:ident ($arg1:expr)) => {
        tryfn!(alias::$fn($arg1), "{:?}")
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
    }
}

pub fn metadata<P: AsRef<Path> + Debug>(path: P) -> Result<Metadata> {
    tryfn!(fs::metadata(path.as_ref()))
}

pub fn mknod<P: AsRef<Path> + Debug>(path: P) -> Result<()> {
    tryfn!(stat::mknod(path.as_ref(), SFlag::S_IFREG, Mode::empty(), 0))
}

pub fn mkdir<P: AsRef<Path> + Debug>(path: P) -> Result<()> {
    tryfn!(alias::mkdir(path.as_ref()))
}

pub fn rmdir<P: AsRef<Path> + Debug>(path: P) -> Result<()> {
    tryfn!(alias::rmdir(path.as_ref()))
}

pub fn chdir<P: AsRef<Path> + Debug>(path: P) -> Result<()> {
    tryfn!(unistd::chdir(path.as_ref()))
}

pub fn touch<P: AsRef<Path> + Debug>(path: P) -> Result<()> {
    if let Some(dir) = path.as_ref().parent() {
        mkdir(dir)?;
    }
    tryfn!(File::create(path.as_ref())).map(|_| ())
}

pub fn write<P: AsRef<Path> + Debug>(path: P, content: &str) -> Result<()> {
    tryfn!(fs::write(path.as_ref(), content.as_bytes()))
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
    let flags = MsFlags::empty();
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
    unsafe { unistd::fork() }.map_err(|err| Error(format!("fork() => {}", err)))
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
    tryfn!(wait::waitpid(pid, Some(WaitPidFlag::empty())))
}

pub fn setrlimit(resource: Resource, limit: Option<u64>) -> Result<()> {
    match limit {
        Some(limit) => tryfn!(resource::setrlimit(resource, limit, limit)),
        None => Ok(()),
    }
}

pub fn sethostname(hostname: &str) -> Result<()> {
    tryfn!(unistd::sethostname(hostname))
}
