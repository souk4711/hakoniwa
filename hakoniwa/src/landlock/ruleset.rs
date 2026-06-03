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

    /// Allow access to files beneath PATH with given mode.
    ///
    /// An allow rule on its own does **not** enable filesystem sandboxing: a
    /// resource is enforced only once it has been turned on explicitly with
    /// [`restrict`](Self::restrict). If [`Resource::FS`] was never restricted,
    /// the paths added here have no effect. This mirrors how namespaces must be
    /// opted into via `Container::unshare` before they take effect. Restrict
    /// first, then allow:
    ///
    /// ```ignore
    /// ruleset.restrict(Resource::FS, CompatMode::Enforce);
    /// ruleset.allow_path("/bin", FsAccess::from_str("r-x").unwrap());
    /// ```
    pub fn allow_path(&mut self, path: &str, mode: FsAccess) -> &mut Self {
        self.add_fs_rule(path, mode)
    }

    /// Allow binding a TCP socket to a local port.
    ///
    /// Takes effect only after [`Resource::NET_TCP_BIND`] is enabled via
    /// [`restrict`](Self::restrict); otherwise the rule is ignored. See
    /// [`allow_path`](Self::allow_path) for the restrict-then-allow pattern.
    pub fn allow_tcp_bind(&mut self, port: u16) -> &mut Self {
        self.add_net_rule(port, NetAccess::TCP_BIND)
    }

    /// Allow connecting an active TCP socket to a remote port.
    ///
    /// Takes effect only after [`Resource::NET_TCP_CONNECT`] is enabled via
    /// [`restrict`](Self::restrict); otherwise the rule is ignored. See
    /// [`allow_path`](Self::allow_path) for the restrict-then-allow pattern.
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
