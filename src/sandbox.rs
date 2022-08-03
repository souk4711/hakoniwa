use serde::Deserialize;

use crate::{Executor, Limits, Namespaces};

#[derive(Default, Deserialize)]
struct SandboxPolicy {
    limits: Limits,
    namespaces: Namespaces,
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
