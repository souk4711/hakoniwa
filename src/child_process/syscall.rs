mod fs {
    use nix::{fcntl, fcntl::OFlag, sys::stat::Mode, unistd};
    use std::{fmt::Debug, fs, path::Path};

    use crate::{defer, tryfn, Result};

    pub fn mkdir<P: AsRef<Path> + Debug>(path: P) -> Result<()> {
        tryfn!(
            unistd::mkdir(path.as_ref(), Mode::S_IRWXU | Mode::S_IRWXG | Mode::S_IRWXO),
            "mkdir({:?}, S_IRWXU | S_IRWXG | S_IRWXO)",
            path
        )
    }

    pub fn rmdir<P: AsRef<Path> + Debug>(path: P) -> Result<()> {
        tryfn!(fs::remove_dir(path.as_ref()), "rmdir({:?})", path)
    }

    pub fn chdir<P: AsRef<Path> + Debug>(path: P) -> Result<()> {
        tryfn!(unistd::chdir(path.as_ref()), "chdir({:?})", path)
    }

    pub fn write<P: AsRef<Path> + Debug>(path: P, content: &str) -> Result<()> {
        let fd = tryfn!(
            fcntl::open(path.as_ref(), OFlag::O_WRONLY, Mode::empty()),
            "open({:?}, O_WRONLY)",
            path
        )?;
        defer! { unistd::close(fd) }

        tryfn!(
            unistd::write(fd, content.as_bytes()),
            "write({:?}, ...)",
            path
        )?;
        Ok(())
    }
}

mod mount {
    use nix::{mount, mount::MntFlags, mount::MsFlags, unistd};
    use std::{fmt::Debug, path::Path};

    use crate::{tryfn, Result};

    const NULL: Option<&'static Path> = None;

    pub fn mount<P1: AsRef<Path> + Debug, P2: AsRef<Path> + Debug>(
        source: P1,
        target: P2,
        flags: MsFlags,
    ) -> Result<()> {
        tryfn!(
            mount::mount(Some(source.as_ref()), target.as_ref(), NULL, flags, NULL),
            "mount({:?}, {:?}, NULL, ..., NULL)",
            source,
            target
        )
    }

    pub fn mount_root() -> Result<()> {
        tryfn!(
            mount::mount(NULL, "/", NULL, MsFlags::MS_PRIVATE | MsFlags::MS_REC, NULL),
            "mount(NULL, {:?}, NULL, MS_PRIVATE | MS_REC, NULL)",
            "/"
        )
    }

    pub fn unmount<P: AsRef<Path> + Debug>(target: P) -> Result<()> {
        tryfn!(
            mount::umount2(target.as_ref(), MntFlags::MNT_DETACH),
            "umount2({:?}, MNT_DETACH)",
            target,
        )
    }

    pub fn pivot_root<P1: AsRef<Path> + Debug, P2: AsRef<Path> + Debug>(
        new_root: P1,
        put_old: P2,
    ) -> Result<()> {
        tryfn!(
            unistd::pivot_root(new_root.as_ref(), put_old.as_ref()),
            "pivot_root({:?}, {:?})",
            new_root,
            put_old
        )
    }
}

mod process {
    use nix::{sched, sched::CloneFlags, sys::resource, sys::resource::Resource, unistd};
    use std::ffi::CStr;

    use crate::{tryfn, Result};

    pub fn unshare(clone_flags: CloneFlags) -> Result<()> {
        tryfn!(sched::unshare(clone_flags), "unshare(...)")
    }

    pub fn execve<SA: AsRef<CStr>, SE: AsRef<CStr>>(
        prog: &CStr,
        argv: &[SA],
        env: &[SE],
    ) -> Result<()> {
        tryfn!(unistd::execve(prog, argv, env), "execve({:?}, ...)", prog)?;
        Ok(())
    }

    pub fn setrlimit(resource: Resource, limit: Option<u64>) -> Result<()> {
        match limit {
            Some(limit) => {
                tryfn!(
                    resource::setrlimit(resource, limit, limit),
                    "setrlimit({:?}, {}, {})",
                    resource,
                    limit,
                    limit
                )
            }
            None => Ok(()),
        }
    }
}

mod sys {
    use nix::unistd;

    use crate::{tryfn, Result};

    pub fn sethostname(hostname: &str) -> Result<()> {
        tryfn!(unistd::sethostname(hostname), "sethostname({:?})", hostname)
    }
}

pub use fs::*;
pub use mount::*;
pub use process::*;
pub use sys::*;
