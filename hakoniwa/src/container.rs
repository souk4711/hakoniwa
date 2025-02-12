use fastrand::alphanumeric;
use std::collections::{HashMap, HashSet};
use std::iter;

use crate::{Command, Namespace, Rlimit};

/// Safe and isolated environment for executing command.
#[derive(Clone)]
pub struct Container {
    pub(crate) namespaces: HashSet<Namespace>,
    pub(crate) hostname: String,
    pub(crate) rlimits: HashMap<Rlimit, (u64, u64)>,
}

impl Container {
    /// Constructor.
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            namespaces: HashSet::new(),
            hostname: iter::repeat_with(alphanumeric).take(8).collect(),
            rlimits: HashMap::new(),
        }
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
        self.hostname = hostname.to_string();
        self
    }

    /// Set resource limit.
    pub fn setrlimit(&mut self, resource: Rlimit, limit: (u64, u64)) -> &mut Self {
        self.rlimits.insert(resource, limit);
        self
    }
}
