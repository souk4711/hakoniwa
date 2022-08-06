mod fs {
    use nix::{fcntl, fcntl::OFlag, sys::stat, sys::stat::Mode, sys::stat::SFlag, unistd};
    use std::{fmt::Debug, fs, fs::Metadata, path::Path};

    use crate::{defer, tryfn, ResultWithError};

    pub fn metadata<P: AsRef<Path> + Debug>(path: P) -> ResultWithError<Metadata> {
        tryfn!(fs::metadata(path.as_ref()), "matadata({:?})", path)
    }

    pub fn mknod<P: AsRef<Path> + Debug>(path: P) -> ResultWithError<()> {
        tryfn!(
            stat::mknod(path.as_ref(), SFlag::S_IFREG, Mode::empty(), 0),
            "mknod({:?})",
            path
        )
    }

    pub fn mkdir<P: AsRef<Path> + Debug>(path: P) -> ResultWithError<()> {
        tryfn!(fs::create_dir_all(path.as_ref()), "mkdir({:?})", path)
    }

    pub fn rmdir<P: AsRef<Path> + Debug>(path: P) -> ResultWithError<()> {
        tryfn!(fs::remove_dir(path.as_ref()), "rmdir({:?})", path)
    }

    pub fn chdir<P: AsRef<Path> + Debug>(path: P) -> ResultWithError<()> {
        tryfn!(unistd::chdir(path.as_ref()), "chdir({:?})", path)
    }

    pub fn touch<P: AsRef<Path> + Debug>(path: P) -> ResultWithError<()> {
        if let Some(dir) = path.as_ref().parent() {
            tryfn!(fs::create_dir_all(dir), "mkdir({:?})", dir)?;
        }
        tryfn!(fs::File::create(path.as_ref()), "touch({:?})", path)?;
        Ok(())
    }

    pub fn write<P: AsRef<Path> + Debug>(path: P, content: &str) -> ResultWithError<()> {
        let flags = OFlag::O_WRONLY;
        let fd = tryfn!(
            fcntl::open(path.as_ref(), flags, Mode::empty()),
            "open({:?}, {:?})",
            path,
            flags
        )?;
        defer! { unistd::close(fd) }

        let content = content.as_bytes();
        tryfn!(unistd::write(fd, content), "write({:?}, ...)", path)?;
        Ok(())
    }
}

mod mount {
    use nix::{mount, mount::MntFlags, mount::MsFlags, unistd};
    use std::{fmt::Debug, path::Path};

    use crate::{tryfn, ResultWithError};

    const NULL: Option<&'static Path> = None;

    pub fn mount<P1: AsRef<Path> + Debug, P2: AsRef<Path> + Debug>(
        source: P1,
        target: P2,
        flags: MsFlags,
    ) -> ResultWithError<()> {
        tryfn!(
            mount::mount(Some(source.as_ref()), target.as_ref(), NULL, flags, NULL),
            "mount({:?}, {:?}, NULL, {:?}, NULL)",
            source,
            target,
            flags
        )
    }

    pub fn mount_root() -> ResultWithError<()> {
        let flags = MsFlags::MS_PRIVATE | MsFlags::MS_REC;
        tryfn!(
            mount::mount(NULL, "/", NULL, flags, NULL),
            "mount(NULL, {:?}, NULL, {:?}, NULL)",
            "/",
            flags
        )
    }

    pub fn mount_proc<P: AsRef<Path> + Debug>(target: P) -> ResultWithError<()> {
        let flags = MsFlags::MS_NOSUID | MsFlags::MS_NODEV | MsFlags::MS_NOEXEC;
        tryfn!(
            mount::mount(NULL, target.as_ref(), Some("proc"), flags, NULL),
            "mount(NULL, {:?}, {:?}, {:?}, NULL)",
            target,
            "proc",
            flags
        )
    }

    pub fn mount_tmpfs<P: AsRef<Path> + Debug>(target: P) -> ResultWithError<()> {
        let flags = MsFlags::empty();
        tryfn!(
            mount::mount(NULL, target.as_ref(), Some("tmpfs"), flags, NULL),
            "mount(NULL, {:?}, {:?}, {:?}, NULL)",
            target,
            "tmpfs",
            flags
        )
    }

    pub fn unmount<P: AsRef<Path> + Debug>(target: P) -> ResultWithError<()> {
        let flags = MntFlags::MNT_DETACH;
        tryfn!(
            mount::umount2(target.as_ref(), flags),
            "umount2({:?}, {:?})",
            target,
            flags
        )
    }

    pub fn pivot_root<P1: AsRef<Path> + Debug, P2: AsRef<Path> + Debug>(
        new_root: P1,
        put_old: P2,
    ) -> ResultWithError<()> {
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

    use crate::{tryfn, ResultWithError};

    pub fn unshare(clone_flags: CloneFlags) -> ResultWithError<()> {
        tryfn!(sched::unshare(clone_flags), "unshare({:?})", clone_flags)
    }

    pub fn execve<SA: AsRef<CStr>, SE: AsRef<CStr>>(
        prog: &CStr,
        argv: &[SA],
        env: &[SE],
    ) -> ResultWithError<()> {
        tryfn!(unistd::execve(prog, argv, env), "execve({:?}, ...)", prog)?;
        Ok(())
    }

    pub fn setrlimit(resource: Resource, limit: Option<u64>) -> ResultWithError<()> {
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

    use crate::{tryfn, ResultWithError};

    pub fn sethostname(hostname: &str) -> ResultWithError<()> {
        tryfn!(unistd::sethostname(hostname), "sethostname({:?})", hostname)
    }
}

pub use fs::*;
pub use mount::*;
pub use process::*;
pub use sys::*;
