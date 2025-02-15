use nix::unistd::{Gid, Uid};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use crate::{Command, IdMap, Mount, MountOptions, Namespace, Rlimit};

/// Safe and isolated environment for executing command.
#[derive(Clone)]
pub struct Container {
    pub(crate) root_dir: Option<PathBuf>,
    pub(crate) root_dir_abspath: PathBuf,
    pub(crate) namespaces: HashSet<Namespace>,
    pub(crate) mounts: Vec<Mount>,
    pub(crate) hostname: Option<String>,
    pub(crate) uidmap: Option<IdMap>,
    pub(crate) gidmap: Option<IdMap>,
    pub(crate) rlimits: HashMap<Rlimit, (u64, u64)>,
}

impl Container {
    /// Constructs a new Container with following setting:
    ///
    /// * a new MOUNT namespace
    /// * a new USER namespace
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        // Create a new mount namespace.
        let mut namespaces = HashSet::new();
        namespaces.insert(Namespace::Mount);

        // Required when creating container as a non-root user.
        //
        // If CLONE_NEWUSER is specified along with other CLONE_NEW* flags in
        // a single clone(2) or unshare(2) call, the user namespace is
        // guaranteed to be created first, giving the child (clone(2)) or
        // caller (unshare(2)) privileges over the remaining namespaces
        // created by the call. Thus, it is possible for an unprivileged
        // caller to specify this combination of flags.
        namespaces.insert(Namespace::User);

        Self {
            root_dir: None,
            root_dir_abspath: PathBuf::new(),
            namespaces,
            mounts: vec![],
            hostname: None,
            uidmap: None,
            gidmap: None,
            rlimits: HashMap::new(),
        }
    }

    /// Use `host_path` as the mount point for the container root fs.
    pub fn root_dir<P: AsRef<Path>>(&mut self, host_path: P) -> &mut Self {
        self.root_dir = Some(host_path.as_ref().to_path_buf());
        self
    }

    /// Mount all subdirectories in `host_path` to the container root fs.
    pub fn rootfs<P: AsRef<Path>>(&mut self, host_path: P) -> &mut Self {
        _ = self.rootfs_imp(host_path);
        self
    }

    /// Containe#rootfs IMP.
    fn rootfs_imp<P: AsRef<Path>>(&mut self, dir: P) -> std::result::Result<(), std::io::Error> {
        let dir = fs::canonicalize(dir)?;
        let mount_options = match dir.to_str() {
            Some("/") => MountOptions::REC,
            _ => MountOptions::empty(),
        };

        let entries = fs::read_dir(&dir)?;
        for entry in entries {
            let path = entry?.path();
            if path.is_dir() {
                let source_abspath = path.to_string_lossy();
                let target_relpath = path.strip_prefix(&dir).unwrap().to_string_lossy();
                let target_abspath = format!("/{}", target_relpath);
                self.bindmount(&source_abspath, &target_abspath, mount_options);
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
    pub fn bindmount(
        &mut self,
        host_path: &str,
        container_path: &str,
        options: MountOptions,
    ) -> &mut Self {
        self.mount(host_path, container_path, MountOptions::BIND | options)
    }

    /// Bind mount the `host_path` on `container_path` with read-only access.
    pub fn bindmount_ro(
        &mut self,
        host_path: &str,
        container_path: &str,
        options: MountOptions,
    ) -> &mut Self {
        self.mount(
            host_path,
            container_path,
            MountOptions::BIND | MountOptions::RDONLY | options,
        )
    }

    /// Mount new tmpfs on `container_path`.
    pub fn tmpfsmount(&mut self, container_path: &str) -> &mut Self {
        self.mount(
            "tmpfs",
            container_path,
            MountOptions::NOSUID | MountOptions::NODEV | MountOptions::NOEXEC,
        )
    }

    /// Mount.
    fn mount(&mut self, host_path: &str, container_path: &str, options: MountOptions) -> &mut Self {
        self.mounts.push(Mount {
            source: host_path.to_string(),
            target: container_path.to_string(),
            options,
        });
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
    pub fn setrlimit(&mut self, resource: Rlimit, limit: (u64, u64)) -> &mut Self {
        self.rlimits.insert(resource, limit);
        self
    }

    /// Constructs a new Command for launching the program at path `program`
    /// within container.
    pub fn command(&self, program: &str) -> Command {
        Command::new(program, self.clone())
    }
}
