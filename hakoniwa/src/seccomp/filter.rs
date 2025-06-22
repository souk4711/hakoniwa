use std::collections::HashSet;

use super::{Action, Arch, ArgCmp, Rule};

/// Represents a filter that allows one to configure actions to take on matched
/// syscalls and furthermore also allows matching on values passed as
/// arguments to syscalls.
#[derive(Clone, Debug)]
pub struct Filter {
    pub(crate) default_action: Action,
    pub(crate) architectures: HashSet<Arch>,
    pub(crate) rules: Vec<Rule>,
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
        let sysname = syscall.to_string();
        let argcmps = vec![];
        self.rules.push(Rule {
            action,
            sysname,
            argcmps,
        });
        self
    }

    /// Adds a single rule for a conditional action on a syscall.
    pub fn add_rule_conditional(
        &mut self,
        action: Action,
        syscall: &str,
        argcmps: &[ArgCmp],
    ) -> &mut Self {
        let sysname = syscall.to_string();
        let argcmps = argcmps.to_vec();
        self.rules.push(Rule {
            action,
            sysname,
            argcmps,
        });
        self
    }

    /// Returns a list of filter rules.
    #[doc(hidden)]
    pub fn get_rules(&self) -> Vec<Rule> {
        self.rules.clone()
    }
}
