use std::collections::HashMap;

use crate::landlock::{FsPerm, FsRule};

/// Landlock ruleset builder.
#[derive(Clone, Default, Debug)]
pub struct Ruleset {
    pub(crate) fs_rules: HashMap<String, FsRule>,
}

impl Ruleset {
    /// Add a new fs rule to the ruleset.
    pub fn add_fs_rule(&mut self, path: &str, perm: FsPerm) -> &mut Self {
        let path = path.to_string();
        self.fs_rules.insert(path.clone(), FsRule { path, perm });
        self
    }

    /// Returns a list of fs rules.
    pub(crate) fn get_fs_rules(&self) -> Vec<&FsRule> {
        let mut values: Vec<_> = self.fs_rules.values().collect();
        values.sort_by(|a, b| a.path.cmp(&b.path));
        values
    }
}
