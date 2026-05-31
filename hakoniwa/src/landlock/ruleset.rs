use std::collections::HashMap;
use std::collections::hash_map::Entry;

use super::{FsAccess, FsRule, NetAccess, NetRule};

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
            Self::NET_TCP_BIND => "tcp.bind",
            Self::NET_TCP_CONNECT => "tcp.connect",
        };
        write!(f, "{r}")
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
    /// DONOT Impose restrictions on resource.
    #[doc(hidden)]
    pub fn unrestrict(&mut self, resource: Resource) -> &mut Self {
        self.restrictions.remove(&resource);
        self
    }

    /// Impose restrictions on resource.
    pub fn restrict(&mut self, resource: Resource, mode: CompatMode) -> &mut Self {
        self.restrictions.insert(resource, mode);
        self
    }

    /// Returns true when the ruleset restricts RESOURCE.
    pub fn is_restricted(&self, resource: Resource) -> bool {
        self.restrictions.contains_key(&resource)
    }

    /// Allow access to files beneath PATH with given mode.
    pub fn allow_path(&mut self, path: &str, mode: FsAccess) -> &mut Self {
        self.add_fs_rule(path, mode)
    }

    /// Allow binding a TCP socket to a local port.
    pub fn allow_tcp_bind(&mut self, port: u16) -> &mut Self {
        self.add_net_rule(port, NetAccess::TCP_BIND)
    }

    /// Allow connecting an active TCP socket to a remote port.
    pub fn allow_tcp_connect(&mut self, port: u16) -> &mut Self {
        self.add_net_rule(port, NetAccess::TCP_CONNECT)
    }

    /// Add a new FS rule to the ruleset.
    fn add_fs_rule(&mut self, path: &str, mode: FsAccess) -> &mut Self {
        let path = path.to_string();
        let rule = FsRule {
            path: path.clone(),
            mode,
        };
        self.fs_rules.insert(path, rule);
        self.restrictions
            .entry(Resource::FS)
            .or_insert(CompatMode::Enforce);
        self
    }

    /// Add a new NET rule to the ruleset.
    fn add_net_rule(&mut self, port: u16, mode: NetAccess) -> &mut Self {
        for e in [NetAccess::TCP_BIND, NetAccess::TCP_CONNECT] {
            let access = mode & e;
            let resource = match access {
                NetAccess::TCP_BIND => Resource::NET_TCP_BIND,
                NetAccess::TCP_CONNECT => Resource::NET_TCP_CONNECT,
                _ => continue,
            };

            self.restrictions
                .entry(resource)
                .or_insert(CompatMode::Enforce);

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allow_path_enables_fs_restriction() {
        let mut ruleset = Ruleset::default();

        ruleset.allow_path("/tmp", FsAccess::R);

        assert_eq!(
            ruleset.restrictions.get(&Resource::FS),
            Some(&CompatMode::Enforce)
        );
        assert!(ruleset.is_restricted(Resource::FS));
    }

    #[test]
    fn allow_path_preserves_existing_fs_compat_mode() {
        let mut ruleset = Ruleset::default();

        ruleset.restrict(Resource::FS, CompatMode::Relax);
        ruleset.allow_path("/tmp", FsAccess::R);

        assert_eq!(
            ruleset.restrictions.get(&Resource::FS),
            Some(&CompatMode::Relax)
        );
    }

    #[test]
    fn allow_tcp_bind_enables_only_bind_restriction() {
        let mut ruleset = Ruleset::default();

        ruleset.allow_tcp_bind(443);

        assert!(ruleset.is_restricted(Resource::NET_TCP_BIND));
        assert!(!ruleset.is_restricted(Resource::NET_TCP_CONNECT));
        assert!(!ruleset.is_restricted(Resource::FS));
    }

    #[test]
    fn allow_tcp_connect_enables_only_connect_restriction() {
        let mut ruleset = Ruleset::default();

        ruleset.allow_tcp_connect(443);

        assert!(ruleset.is_restricted(Resource::NET_TCP_CONNECT));
        assert!(!ruleset.is_restricted(Resource::NET_TCP_BIND));
    }

    #[test]
    fn net_rule_preserves_existing_compat_mode() {
        let mut ruleset = Ruleset::default();

        ruleset.restrict(Resource::NET_TCP_CONNECT, CompatMode::Relax);
        ruleset.allow_tcp_connect(443);

        assert_eq!(
            ruleset.restrictions.get(&Resource::NET_TCP_CONNECT),
            Some(&CompatMode::Relax)
        );
    }
}
