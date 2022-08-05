use serde::Deserialize;
use std::str as Str;

use crate::{Embed, Executor, Limits, Mount, Namespaces, ResultWithError};

#[derive(Deserialize, Default)]
pub struct SandboxPolicy {
    #[serde(default)]
    limits: Limits,
    #[serde(default)]
    mounts: Vec<Mount>,
}

impl SandboxPolicy {
    pub fn from_str(data: &str) -> ResultWithError<Self> {
        let policy: Self = toml::from_str(data)?;
        Ok(policy)
    }

    #[allow(non_snake_case)]
    pub(crate) fn KISS_POLICY() -> Self {
        let f = Embed::get("KISS-policy.toml").unwrap();
        let data = Str::from_utf8(f.data.as_ref()).unwrap();
        Self::from_str(data).unwrap()
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
