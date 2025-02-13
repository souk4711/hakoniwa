use nix::unistd::{Gid, Uid};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use crate::{Command, IdMap, Namespace, Rlimit};

/// Safe and isolated environment for executing command.
#[derive(Clone, Default)]
pub struct Container {
    pub(crate) root_dir: PathBuf,
    pub(crate) namespaces: HashSet<Namespace>,
    pub(crate) hostname: Option<String>,
    pub(crate) uidmap: Option<IdMap>,
    pub(crate) gidmap: Option<IdMap>,
    pub(crate) rlimits: HashMap<Rlimit, (u64, u64)>,
}

impl Container {
    /// Constructor.
    pub fn new() -> Self {
        Self::default()
    }

    /// Use `dir` as the mount point for the container root fs.
    pub fn root_dir<P: AsRef<Path>>(&mut self, dir: P) -> &mut Self {
        self.root_dir = dir.as_ref().to_path_buf();
        self
    }

    /// Constructs a new Command for launching the program at path `program`
    /// within container.
    pub fn command(&self, program: &str) -> Command {
        Command::new(program, self.clone())
    }

    /// Unshare a namespace.
    pub fn unshare_namespace(&mut self, namespace: Namespace) -> &mut Self {
        self.namespaces.insert(namespace);
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
}
