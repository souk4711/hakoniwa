use std::collections::HashSet;

use crate::seccomp::{Action, Arch, ArgCmp};

/// Represents a filter that allows one to configure actions to take on matched
/// syscalls and furthermore also allows matching on values passed as
/// arguments to syscalls.
#[derive(Clone)]
pub struct Filter {
    pub(crate) default_action: Action,
    pub(crate) architectures: HashSet<Arch>,
    pub(crate) rules: Vec<(Action, String, Vec<ArgCmp>)>,
}

impl Filter {
    /// Creates and returns a new filter.
    pub fn new(default_action: Action) -> Self {
        Self {
            default_action,
            architectures: HashSet::new(),
            rules: vec![],
        }
    }

    /// Adds an architecture to the filter.
    pub fn add_arch(&mut self, arch: Arch) -> &mut Self {
        self.architectures.insert(arch);
        self
    }

    /// Adds a single rule for an unconditional action on a syscall.
    pub fn add_rule(&mut self, action: Action, syscall: &str) -> &mut Self {
        let syscall = syscall.to_string();
        let argcmps = vec![];
        self.rules.push((action, syscall, argcmps));
        self
    }

    /// Adds a single rule for a conditional action on a syscall.
    pub fn add_rule_conditional(
        &mut self,
        action: Action,
        syscall: &str,
        argcmps: &[ArgCmp],
    ) -> &mut Self {
        let syscall = syscall.to_string();
        let argcmps = argcmps.to_vec();
        self.rules.push((action, syscall, argcmps));
        self
    }
}
