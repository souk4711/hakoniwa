use nix::sched::CloneFlags;
use nix::unistd::{Gid, Uid};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use crate::{Command, FsOperation, IdMap, Mount, MountOptions, Namespace, Network, Rlimit, Runctl};

/// Safe and isolated environment for executing command.
///
/// A default environment can be generated using [Container::new], which will
/// unshare necessary namespaces. Then use [bindmount_ro] or [bindmount_rw] to
/// mount directories to the container root.
///
/// ```no_run
/// use hakoniwa::Container;
///
/// let mut container = Container::new();
/// container.bindmount_ro("/bin", "/bin")
///     .bindmount_ro("/lib", "/lib")
///     .bindmount_ro("/lib64", "/lib64")
///     .bindmount_ro("/usr", "/usr");
/// ```
///
/// And now, we can execute [Command] in the container.
///
/// ```no_run
/// # let mut container = hakoniwa::Container::new();
/// # container.bindmount_ro("/bin", "/bin")
/// #    .bindmount_ro("/lib", "/lib")
/// #    .bindmount_ro("/lib64", "/lib64")
/// #    .bindmount_ro("/usr", "/usr");
/// let mut command = container.command("/bin/echo");
/// let output = command.arg("hello")
///     .output()
///     .expect("failed to execute process witnin container");
/// ```
///
/// [bindmount_ro]: Container::bindmount_ro
/// [bindmount_rw]: Container::bindmount_rw
#[derive(Clone, Debug)]
pub struct Container {
    pub(crate) namespaces: HashSet<Namespace>,
    pub(crate) rootdir: Option<PathBuf>,
    pub(crate) rootdir_abspath: PathBuf,
    mounts: HashMap<String, Mount>,
    fs_operations: HashMap<String, FsOperation>,
    pub(crate) uidmaps: Option<Vec<IdMap>>,
    pub(crate) gidmaps: Option<Vec<IdMap>>,
    pub(crate) hostname: Option<String>,
    pub(crate) network: Option<Network>,
    pub(crate) rlimits: HashMap<Rlimit, (u64, u64)>,
    #[cfg(feature = "landlock")]
    pub(crate) landlock_ruleset: Option<crate::landlock::Ruleset>,
    #[cfg(feature = "seccomp")]
    pub(crate) seccomp_filter: Option<crate::seccomp::Filter>,
    pub(crate) runctl: HashSet<Runctl>,
}

impl Container {
    /// Constructs a new Container with following steps:
    ///
    /// - Create a new MOUNT namespace
    /// - Create a new USER namespace and map current user to itself
    /// - Create a new PID namespace and mount a new procfs on `/proc`
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let mut container = Self::empty();

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
        container.uidmap(Uid::current().as_raw());
        container.gidmap(Gid::current().as_raw());

        // A /proc filesystem shows (in the /proc/pid directories) only
        // processes visible in the PID namespace of the process that
        // performed the mount, even if the /proc filesystem is viewed from
        // processes in other namespaces.
        //
        // After creating a new PID namespace, it is useful for the child to
        // change its root directory and mount a new procfs instance at /proc
        // so that tools such as ps(1) work correctly.
        container.unshare(Namespace::Pid);
        container.procfsmount("/proc");

        // Self
        container
    }

    /// Constructs a new Container with a completely empty environment.
    pub fn empty() -> Self {
        Self {
            namespaces: HashSet::new(),
            rootdir: None,
            rootdir_abspath: PathBuf::new(),
            mounts: HashMap::new(),
            fs_operations: HashMap::new(),
            uidmaps: None,
            gidmaps: None,
            hostname: None,
            network: None,
            rlimits: HashMap::new(),
            #[cfg(feature = "landlock")]
            landlock_ruleset: None,
            #[cfg(feature = "seccomp")]
            seccomp_filter: None,
            runctl: HashSet::new(),
        }
    }

    /// DONOT Create a new namespace.
    #[doc(hidden)]
    pub fn share(&mut self, namespace: Namespace) -> &mut Self {
        self.namespaces.remove(&namespace);
        self
    }

    /// Create a new namespace.
    pub fn unshare(&mut self, namespace: Namespace) -> &mut Self {
        self.namespaces.insert(namespace);
        self
    }

    /// Use `host_path` as the mount point for the container root fs.
    ///
    /// By default the mount point is a tmpdir, and will be automatically
    /// cleaned up when the last process exits.
    ///
    /// This method is mainly useful if you set it to a directory that
    /// contains a file system hierarchy, and want chroot into it.
    ///
    /// # Panics
    ///
    /// Panics if `host_path` does not exists.
    ///
    /// # Caveats
    ///
    /// Some empty directories/files that were used as mount point targets
    /// may be left behind even when the last process exits.
    pub fn rootdir<P: AsRef<Path>>(&mut self, host_path: P) -> &mut Self {
        let host_path = fs::canonicalize(&host_path).unwrap();
        self.rootdir = Some(host_path);
        self
    }

    /// Bind mount all subdirectories in `host_path` to the container with
    /// read-only access in new MOUNT namespace.
    ///
    /// # Panics
    ///
    /// Panics if `host_path` does not exists.
    ///
    /// # Caveats
    ///
    /// When use `/` as rootfs, it only mount following subdirectories: `/bin`,
    /// `/etc`, `/lib`, `/lib64`, `/lib32`, `/sbin`, `/usr`.
    pub fn rootfs<P: AsRef<Path>>(&mut self, host_path: P) -> &mut Self {
        _ = self.rootfs_imp(host_path);
        self
    }

    /// Container#rootfs IMP.
    fn rootfs_imp<P: AsRef<Path>>(&mut self, dir: P) -> std::result::Result<(), std::io::Error> {
        let mut entries = vec![];
        if dir.as_ref() == PathBuf::from("/") {
            for entry in ["/bin", "/etc", "/lib", "/lib64", "/lib32", "/sbin", "/usr"] {
                entries.push(PathBuf::from(entry));
            }
        } else {
            let dir = fs::canonicalize(&dir).unwrap();
            for entry in fs::read_dir(&dir)? {
                entries.push(entry?.path());
            }
        }

        for entry in entries {
            if !entry.is_dir() {
                continue;
            }

            let container_relpath = entry.strip_prefix(&dir).unwrap().to_string_lossy();
            let container_abspath = format!("/{container_relpath}");
            if container_abspath == "/proc" {
                continue;
            }

            if entry.is_symlink() {
                let original = fs::read_link(&entry)?;
                let original = original.as_path().to_string_lossy();
                self.symlink(&original, &container_abspath);
            } else {
                let host_abspath = entry.to_string_lossy();
                self.bindmount_ro(&host_abspath, &container_abspath);
            }
        }
        Ok(())
    }

    /// Bind mount the `host_path` on `container_path` with read-only access in new MOUNT namespace.
    pub fn bindmount_ro(&mut self, host_path: &str, container_path: &str) -> &mut Self {
        let flags =
            MountOptions::BIND | MountOptions::REC | MountOptions::NOSUID | MountOptions::RDONLY;
        self.mount(host_path, container_path, "", flags)
    }

    /// Bind mount the `host_path` on `container_path` with read-write access in new MOUNT namespace.
    pub fn bindmount_rw(&mut self, host_path: &str, container_path: &str) -> &mut Self {
        let flags = MountOptions::BIND | MountOptions::REC | MountOptions::NOSUID;
        self.mount(host_path, container_path, "", flags)
    }

    /// Mount new devfs on `container_path` in new MOUNT namespace.
    ///
    /// # Caveats
    ///
    /// This is not a real linux filesystem type. It just bind mount a minimal set
    /// of device files in `container_path`, such as `/dev/null`.
    pub fn devfsmount(&mut self, container_path: &str) -> &mut Self {
        let flags = MountOptions::empty();
        self.mount("devfs", container_path, "devfs", flags)
    }

    /// Mount new tmpfs on `container_path` in new MOUNT namespace.
    pub fn tmpfsmount(&mut self, container_path: &str) -> &mut Self {
        let flags = MountOptions::NOSUID | MountOptions::NODEV;
        self.mount("tmpfs", container_path, "tmpfs", flags)
    }

    /// Mount new procfs on `container_path` in new MOUNT namespace.
    pub fn procfsmount(&mut self, container_path: &str) -> &mut Self {
        let flags = MountOptions::NOSUID | MountOptions::NODEV | MountOptions::NOEXEC;
        self.mount("proc", container_path, "proc", flags)
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

    /// Creates a new file with `contents` on the filesystem in new MOUNT namespace.
    pub fn file(&mut self, target: &str, contents: &str) -> &mut Self {
        let target = target.to_string();
        let contents = contents.to_string();
        let op = crate::unshare::FsWriteFile {
            target: target.clone(),
            contents,
        };
        self.fs_operations.insert(target, op.into());
        self
    }

    /// Creates a new dir with `mode` in new MOUNT namespace.
    pub fn dir(&mut self, target: &str, mode: u32) -> &mut Self {
        let target = target.to_string();
        let op = crate::unshare::FsMakeDir {
            target: target.clone(),
            mode,
        };
        self.fs_operations.insert(target, op.into());
        self
    }

    /// Creates a new symbolic link on the filesystem in new MOUNT namespace.
    pub fn symlink(&mut self, original: &str, link: &str) -> &mut Self {
        let original = original.to_string();
        let link = link.to_string();
        let op = crate::unshare::FsMakeSymlink {
            original,
            link: link.clone(),
        };
        self.fs_operations.insert(link, op.into());
        self
    }

    /// Map current user to uid in new USER namespace.
    ///
    /// This is a shorthand for `uidmaps(vec![(uid, Uid::current().as_raw(), 1)])`
    pub fn uidmap(&mut self, uid: u32) -> &mut Self {
        self.uidmaps(vec![(uid, Uid::current().as_raw(), 1)]);
        self
    }

    /// Map current group to gid in new USER namespace.
    ///
    /// This is a shorthand for `gidmaps(vec![(gid, Gid::current().as_raw(), 1)])`
    pub fn gidmap(&mut self, gid: u32) -> &mut Self {
        self.gidmaps(vec![(gid, Gid::current().as_raw(), 1)]);
        self
    }

    /// Create new UID maps in new USER namespace.
    pub fn uidmaps(&mut self, idmaps: Vec<(u32, u32, u32)>) -> &Self {
        self.uidmaps = Self::idmaps(idmaps);
        self
    }

    /// Create new GID maps in new USER namespace.
    pub fn gidmaps(&mut self, idmaps: Vec<(u32, u32, u32)>) -> &Self {
        self.gidmaps = Self::idmaps(idmaps);
        self
    }

    /// From Vec<(u32, u32, u32)> to Vec<IDMap>.
    fn idmaps(idmaps: Vec<(u32, u32, u32)>) -> Option<Vec<IdMap>> {
        if idmaps.is_empty() {
            return None;
        }

        let idmaps = idmaps
            .iter()
            .map(|e| IdMap {
                container_id: e.0,
                host_id: e.1,
                size: e.2,
            })
            .collect::<Vec<IdMap>>();
        Some(idmaps)
    }

    /// Changes the hostname in the new UTS namespace.
    pub fn hostname(&mut self, hostname: &str) -> &mut Self {
        self.hostname = Some(hostname.to_string());
        self
    }

    /// Change the network mode in new Network namespace.
    pub fn network<T: Into<Network>>(&mut self, network: T) -> &mut Self {
        self.network = Some(network.into());
        self
    }

    /// Set resource limit.
    pub fn setrlimit(&mut self, resource: Rlimit, soft_limit: u64, hard_limit: u64) -> &mut Self {
        self.rlimits.insert(resource, (soft_limit, hard_limit));
        self
    }

    /// Set landlock ruleset.
    #[cfg(feature = "landlock")]
    pub fn landlock_ruleset(&mut self, ruleset: crate::landlock::Ruleset) -> &mut Self {
        self.landlock_ruleset = Some(ruleset);
        self
    }

    /// Set seccomp filter.
    #[cfg(feature = "seccomp")]
    pub fn seccomp_filter(&mut self, filter: crate::seccomp::Filter) -> &mut Self {
        self.seccomp_filter = Some(filter);
        self
    }

    /// Manipulates various aspects of the behavior of the container.
    pub fn runctl(&mut self, ctl: Runctl) -> &mut Self {
        self.runctl.insert(ctl);
        self
    }

    /// Constructs a new Command for launching the program at path `program`
    /// within container.
    pub fn command(&self, program: &str) -> Command {
        Command::new(program, self.clone())
    }

    /// Returns Namespaces in CloneFlags format.
    pub(crate) fn get_namespaces_clone_flags(&self) -> CloneFlags {
        let mut flags = CloneFlags::empty();
        for flag in &self.namespaces {
            flags.insert(flag.to_clone_flag())
        }
        flags
    }

    /// Returns a list of Mount sorted by target path.
    pub(crate) fn get_mounts(&self) -> Vec<&Mount> {
        let mut values: Vec<_> = self.mounts.values().collect();
        values.sort_by(|a, b| a.target.cmp(&b.target));
        values
    }

    /// Returns a Mount whose fstype is proc.
    pub(crate) fn get_mount_newproc(&self) -> Option<&Mount> {
        let values = self.get_mounts();
        let value = values.iter().find(|mount| mount.fstype == "proc");
        value.copied()
    }

    /// Returns a list of FS Operation sorted by target path.
    pub(crate) fn get_fs_operations(&self) -> Vec<&FsOperation> {
        let mut keys: Vec<_> = self.fs_operations.keys().collect();
        keys.sort();
        keys.clone()
            .into_iter()
            .filter_map(|k| self.fs_operations.get(k))
            .collect()
    }

    /// Returns setup operations in bit flags.
    pub(crate) fn get_mainp_setup_operations(&self) -> u8 {
        let mut operations = 0;
        if self.needs_mainp_setup_network() {
            operations |= crate::runc::SETUP_NETWORK;
        }
        if self.needs_mainp_setup_ugidmap() {
            operations |= crate::runc::SETUP_UGIDMAP;
        }
        operations
    }

    /// Returns true if the container needs the main process to setup
    /// the network.
    pub(crate) fn needs_mainp_setup_network(&self) -> bool {
        if !self.namespaces.contains(&Namespace::Network) {
            return false;
        }
        self.network.is_some()
    }

    /// Returns true if the container needs the main process to setup
    /// the [ug]idmap.
    pub(crate) fn needs_mainp_setup_ugidmap(&self) -> bool {
        if !self.namespaces.contains(&Namespace::User) {
            return false;
        }
        let uidmaps = self.uidmaps.clone().unwrap_or_default();
        let gidmaps = self.gidmaps.clone().unwrap_or_default();
        uidmaps.len() > 1 || gidmaps.len() > 1
    }

    /// Returns true if the container needs the child process to stop
    /// the internal process at exit.
    pub(crate) fn needs_childp_traceexit(&self) -> bool {
        self.runctl.contains(&Runctl::GetProcPidSmapsRollup)
            || self.runctl.contains(&Runctl::GetProcPidStatus)
    }
}
