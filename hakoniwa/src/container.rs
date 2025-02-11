use nix::sched::CloneFlags;
use std::collections::{HashMap, HashSet};

use crate::{Command, Namespace, Rlimit};

/// Safe and isolated environment for executing command.
#[derive(Clone, Default)]
pub struct Container {
    namespaces: HashSet<Namespace>,
    pub(crate) rlimits: HashMap<Rlimit, (u64, u64)>,
}

impl Container {
    /// Constructor.
    pub fn new() -> Self {
        Default::default()
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

    ///
    pub(crate) fn namespaces_to_clone_flags(&self) -> CloneFlags {
        let mut flags = CloneFlags::empty();
        for flag in &self.namespaces {
            flags.insert(flag.to_clone_flag())
        }
        flags
    }

    /// Set resource limit.
    pub fn setrlimit(&mut self, rlimit: Rlimit, val: (u64, u64)) -> &mut Self {
        self.rlimits.insert(rlimit, val);
        self
    }
}
