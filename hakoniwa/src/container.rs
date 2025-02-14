use fastrand::alphanumeric;
use nix::unistd::{Gid, Uid};
use std::collections::{HashMap, HashSet};
use std::env;
use std::iter;
use std::path::{Path, PathBuf};

use crate::{Command, IdMap, Mount, MountOptions, Namespace, Rlimit};

/// Safe and isolated environment for executing command.
#[derive(Clone)]
pub struct Container {
    pub(crate) root_dir: PathBuf,
    pub(crate) namespaces: HashSet<Namespace>,
    pub(crate) mounts: Vec<Mount>,
    pub(crate) hostname: Option<String>,
    pub(crate) uidmap: Option<IdMap>,
    pub(crate) gidmap: Option<IdMap>,
    pub(crate) rlimits: HashMap<Rlimit, (u64, u64)>,
}


impl Container {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let name: String = iter::repeat_with(alphanumeric).take(8).collect();
        let name = format!("hakoniwa-{}", name);
        let root_dir = env::temp_dir().join(name);
        _ = std::fs::create_dir_all(&root_dir);

        let mut namespaces = HashSet::new();
        namespaces.insert(Namespace::Mount);
        namespaces.insert(Namespace::User);
        namespaces.insert(Namespace::Pid);

        Self {
            root_dir,
            namespaces,
            mounts: vec![],
            hostname: None,
            uidmap: None,
            gidmap: None,
            rlimits: HashMap::new(),
        }
    }

    ///
    pub fn rootfs<P: AsRef<Path>>(&mut self, _dir: P) -> &mut Self {
        self.bindmount("/bin", "/bin", MountOptions::REC);
        self.bindmount("/lib", "/lib", MountOptions::REC);
        self.bindmount("/lib64", "/lib64", MountOptions::REC);
        self.bindmount("/usr", "/usr", MountOptions::REC);
        self
    }

    /// Use `dir` as the mount point for the container root fs.
    pub fn root_dir<P: AsRef<Path>>(&mut self, dir: P) -> &mut Self {
        self.root_dir = dir.as_ref().to_path_buf();
        self
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
