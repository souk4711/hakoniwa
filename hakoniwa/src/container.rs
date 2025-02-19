use nix::unistd::{Gid, Uid};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use crate::{Command, IdMap, Mount, MountOptions, Namespace, Rlimit};

/// Safe and isolated environment for executing command.
///
/// A default environment can be generated using [Container::new], which will
/// unshare necessary namespaces. Then use [bindmount] or [bindmount_ro] to
/// mount directories to the container root.
///
/// ```no_run
/// use hakoniwa::Container;
///
/// let mut container = Container::new();
/// container.bindmount_ro("/bin", "/bin")
///     .bindmount_ro("/lib", "lib");
/// ```
///
/// And now, we can execute [Command] in the container.
///
/// ```no_run
/// # let mut container = hakoniwa::Container::new();
/// # container.bindmount_ro("/bin", "/bin")
/// #    .bindmount_ro("/lib", "lib");
/// let mut command = container.command("/bin/echo");
/// let output = command.arg("hello")
///     .output()
///     .expect("failed to execute process witnin container");
/// ```
///
/// [bindmount]: Container::bindmount
/// [bindmount_ro]: Container::bindmount_ro
#[derive(Clone)]
pub struct Container {
    pub(crate) root_dir: Option<PathBuf>,
    pub(crate) root_dir_abspath: PathBuf,
    pub(crate) namespaces: HashSet<Namespace>,
    pub(crate) mounts: HashMap<String, Mount>,
    pub(crate) hostname: Option<String>,
    pub(crate) uidmap: Option<IdMap>,
    pub(crate) gidmap: Option<IdMap>,
    pub(crate) rlimits: HashMap<Rlimit, (u64, u64)>,
}

impl Container {
    /// Constructs a new Container with following steps:
    ///
    /// * Create a new MOUNT namespace
    /// * Create a new USER namespace
    /// * Create a new PID namespace and mount a new procfs on `/proc`
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let mut container = Self {
            root_dir: None,
            root_dir_abspath: PathBuf::new(),
            namespaces: HashSet::new(),
            mounts: HashMap::new(),
            hostname: None,
            uidmap: None,
            gidmap: None,
            rlimits: HashMap::new(),
        };

        // Create a new MOUNT namespace.
        container.unshare(Namespace::Mount);

        // Required when creating container as a non-root user.
        //
        // If CLONE_NEWUSER is specified along with other CLONE_NEW* flags in
        // a single clone(2) or unshare(2) call, the user namespace is
        // guaranteed to be created first, giving the child (clone(2)) or
        // caller (unshare(2)) privileges over the remaining namespaces
        // created by the call. Thus, it is possible for an unprivileged
        // caller to specify this combination of flags.
        container.unshare(Namespace::User);

        // A /proc filesystem shows (in the /proc/pid directories) only
        // processes visible in the PID namespace of the process that
        // performed the mount, even if the /proc filesystem is viewed from
        // processes in other namespaces.
        //
        // After creating a new PID namespace, it is useful for the child to
        // change its root directory and mount a new procfs instance at /proc
        // so that tools such as ps(1) work correctly.
        container.unshare(Namespace::Pid);
        container.procfsmount();

        // Self
        container
    }

    /// Use `host_path` as the mount point for the container root fs.
    ///
    /// # Panics
    ///
    /// Panics if `host_path` does not exists.
    pub fn root_dir<P: AsRef<Path>>(&mut self, host_path: P) -> &mut Self {
        let host_path = fs::canonicalize(&host_path).unwrap();
        self.root_dir = Some(host_path);
        self
    }

    /// Mount all subdirectories in `host_path` to the container root fs as
    /// a read-only file system.
    ///
    /// # Panics
    ///
    /// Panics if `host_path` does not exists.
    ///
    /// # Caveats
    ///
    /// When use `/` as rootfs, it only mount following subdirectories: `/bin`,
    /// `/etc`, `/lib`, `/lib64`, `/sbin`, `/usr`.
    pub fn rootfs<P: AsRef<Path>>(&mut self, host_path: P) -> &mut Self {
        _ = self.rootfs_imp(host_path);
        self
    }

    /// Container#rootfs IMP.
    fn rootfs_imp<P: AsRef<Path>>(&mut self, dir: P) -> std::result::Result<(), std::io::Error> {
        // Local rootfs.
        if dir.as_ref() == PathBuf::from("/") {
            let path = ["/bin", "/etc", "/lib", "/lib64", "/sbin", "/usr"];
            let paths: Vec<_> = path
                .into_iter()
                .filter(|path| Path::new(path).is_dir())
                .collect();
            for path in paths {
                self.bindmount_ro(path, path);
            }
            return Ok(());
        }

        // Customized rootfs.
        let dir = fs::canonicalize(dir).unwrap();
        let entries = fs::read_dir(&dir)?;
        for entry in entries {
            let path = entry?.path();
            if path.is_dir() {
                let source_abspath = path.to_string_lossy();
                let target_relpath = path.strip_prefix(&dir).unwrap().to_string_lossy();
                let target_abspath = format!("/{}", target_relpath);
                self.bindmount_ro(&source_abspath, &target_abspath);
            }
        }
        Ok(())
    }

    /// Create a new namespace.
    pub fn unshare(&mut self, namespace: Namespace) -> &mut Self {
        self.namespaces.insert(namespace);
        self
    }

    /// Bind mount the `host_path` on `container_path`.
    pub fn bindmount(&mut self, host_path: &str, container_path: &str) -> &mut Self {
        self.mount(
            host_path,
            container_path,
            "",
            MountOptions::BIND | MountOptions::REC,
        )
    }

    /// Bind mount the `host_path` on `container_path` with read-only access.
    pub fn bindmount_ro(&mut self, host_path: &str, container_path: &str) -> &mut Self {
        self.mount(
            host_path,
            container_path,
            "",
            MountOptions::BIND | MountOptions::REC | MountOptions::RDONLY,
        )
    }

    /// Mount new tmpfs on `container_path`.
    pub fn tmpfsmount(&mut self, container_path: &str) -> &mut Self {
        self.mount(
            "",
            container_path,
            "tmpfs",
            MountOptions::NOSUID | MountOptions::NODEV | MountOptions::NOEXEC,
        )
    }

    /// Mount new procfs on `/proc`.
    pub fn procfsmount(&mut self) -> &mut Self {
        self.mount(
            "",
            "/proc",
            "procfs",
            MountOptions::NOSUID | MountOptions::NODEV | MountOptions::NOEXEC,
        )
    }

    /// Mount.
    #[doc(hidden)]
    pub fn mount(
        &mut self,
        host_path: &str,
        container_path: &str,
        fstype: &str,
        options: MountOptions,
    ) -> &mut Self {
        let source = host_path.to_string();
        let target = container_path.to_string();
        let fstype = fstype.to_string();
        self.mounts.insert(
            target.clone(),
            Mount {
                source,
                target,
                fstype,
                options,
            },
        );
        self
    }

    /// Changes the hostname in the new UTS namespace.
    pub fn hostname(&mut self, hostname: &str) -> &mut Self {
        self.hostname = Some(hostname.to_string());
        self
    }

    /// Map current user to uid in new USER namespace.
    pub fn uidmap(&mut self, uid: u32) -> &mut Self {
        self.uidmap = Some(IdMap {
            container_id: uid,
            host_id: Uid::current().as_raw(),
            size: 1,
        });
        self
    }

    /// Map current group to gid in new USER namespace.
    pub fn gidmap(&mut self, gid: u32) -> &mut Self {
        self.gidmap = Some(IdMap {
            container_id: gid,
            host_id: Gid::current().as_raw(),
            size: 1,
        });
        self
    }

    /// Set resource limit.
    pub fn setrlimit(&mut self, resource: Rlimit, soft_limit: u64, hard_limit: u64) -> &mut Self {
        self.rlimits.insert(resource, (soft_limit, hard_limit));
        self
    }

    /// Constructs a new Command for launching the program at path `program`
    /// within container.
    pub fn command(&self, program: &str) -> Command {
        Command::new(program, self.clone())
    }
}
