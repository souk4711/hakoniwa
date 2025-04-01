use std::collections::hash_map::Entry;
use std::collections::HashMap;

use crate::landlock::*;

/// Resource type.
#[allow(non_camel_case_types)]
#[derive(Hash, Eq, PartialEq, Clone, Copy, Debug)]
pub enum Resource {
    FS,
    NET_TCP_BIND,
    NET_TCP_CONNECT,
}

impl std::fmt::Display for Resource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let r = match self {
            Self::FS => "fs",
            Self::NET_TCP_BIND => "net.tcp.bind",
            Self::NET_TCP_CONNECT => "net.tcp.connect",
        };
        write!(f, "{}", r)
    }
}

/// Compatibility mode.
#[derive(Hash, Eq, PartialEq, Clone, Copy, Debug)]
pub enum CompatMode {
    Enforce,
    Relax,
}

/// Landlock ruleset builder.
#[derive(Clone, Default, Debug)]
pub struct Ruleset {
    pub(crate) restrictions: HashMap<Resource, CompatMode>,
    pub(crate) fs_rules: HashMap<String, FsRule>,
    pub(crate) net_rules: HashMap<Resource, Vec<NetRule>>,
}

impl Ruleset {
    /// Impose restrictions on resource.
    pub fn restrict(&mut self, resource: Resource, mode: CompatMode) -> &mut Self {
        self.restrictions.insert(resource, mode);
        self
    }

    /// Add a new FS rule to the ruleset.
    pub fn add_fs_rule(&mut self, path: &str, mode: FsAccess) -> &mut Self {
        let path = path.to_string();
        let rule = FsRule {
            path: path.clone(),
            mode,
        };
        self.fs_rules.insert(path, rule);
        self
    }

    /// Add a new NET rule to the ruleset.
    pub fn add_net_rule(&mut self, port: u16, mode: NetAccess) -> &mut Self {
        for e in [NetAccess::TCP_BIND, NetAccess::TCP_CONNECT] {
            let access = mode & e;
            let resource = match access {
                NetAccess::TCP_BIND => Resource::NET_TCP_BIND,
                NetAccess::TCP_CONNECT => Resource::NET_TCP_CONNECT,
                _ => continue,
            };

            let rule = NetRule { port, access };
            match self.net_rules.entry(resource) {
                Entry::Vacant(e) => {
                    e.insert(vec![rule]);
                }
                Entry::Occupied(mut e) => {
                    e.get_mut().push(rule);
                }
            };
        }
        self
    }

    /// Returns a list of fs rules.
    pub(crate) fn get_fs_rules(&self) -> Vec<&FsRule> {
        let mut values: Vec<_> = self.fs_rules.values().collect();
        values.sort_by(|a, b| a.path.cmp(&b.path));
        values
    }
}
