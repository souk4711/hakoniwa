use serde::Deserialize;

use crate::executor::Executor;
use crate::limits::Limits;
use crate::namespaces::Namespaces;

#[derive(Deserialize)]
struct SandboxPolicy {
    limits: Limits,
    namespaces: Namespaces,
}

// impl trait Default
impl Default for SandboxPolicy {
    fn default() -> Self {
        SandboxPolicy {
            limits: Limits {
                r#as: Some(512 * 1024 * 1024), // 512 MiB
                core: Some(0),                 // no core dumps
                cpu: None,                     // unlimited
                fsize: Some(16 * 1024 * 1024), // 16 MiB
                nofile: Some(256),             // 256
            },
            namespaces: Namespaces {
                ns: Some(true),   // create new mount namespace
                uts: Some(true),  // create new uts namespace
                ipc: Some(true),  // create new ipc namespace
                pid: Some(true),  // create new pid namespace
                net: Some(true),  // create new network namespace
                user: Some(true), // create new user namespace
            },
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

    pub fn command<T: AsRef<str>>(&self, prog: &str, argv: &[T]) -> Executor {
        let mut executor = Executor::new(prog, argv);
        executor
            .limits(self.policy.limits.clone())
            .namespaces(self.policy.namespaces.clone());
        executor
    }
}
