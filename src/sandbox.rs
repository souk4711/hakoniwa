use serde::Deserialize;
use std::path::Path;

use crate::{Executor, Limits, Mount, MountKind, Namespaces};

#[derive(Deserialize, Default)]
pub struct SandboxPolicy {
    limits: Limits,
    mounts: Vec<Mount>,
}

impl SandboxPolicy {
    pub fn default() -> Self {
        SandboxPolicy {
            limits: Limits::default(),
            mounts: [
                ("/bin", "/bin"),
                ("/lib", "/lib"),
                ("/lib64", "/lib64"),
                ("/usr", "/usr"),
            ]
            .iter()
            .filter_map(|(host_path, container_path)| {
                if Path::new(&host_path).exists() {
                    Some(Mount::new(host_path, container_path, MountKind::RoBind))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>(),
        }
    }
}

#[derive(Default)]
pub struct Sandbox {
    policy: SandboxPolicy,
}

impl Sandbox {
    pub fn new() -> Self {
        Sandbox {
            ..Default::default()
        }
    }

    pub fn with_policy(&mut self, policy: SandboxPolicy) -> &mut Self {
        self.policy = policy;
        self
    }

    pub fn command<SA: AsRef<str>>(&self, prog: &str, argv: &[SA]) -> Executor {
        let mut executor = Executor::new(prog, argv);
        executor
            .limits(self.policy.limits.clone())
            .namespaces(Namespaces::default())
            .mounts(self.policy.mounts.clone());
        executor
    }
}
