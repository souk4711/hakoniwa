use handlebars::Handlebars;
use lazy_static::lazy_static;
use serde::Deserialize;
use std::{collections::HashMap, str};

use crate::{contrib, Embed, Executor, Limits, Mount, Namespaces, Result};

lazy_static! {
    static ref SANDBOX_POLICY_HANDLEBARS: Handlebars<'static> = {
        let mut handlebars = Handlebars::new();
        handlebars.register_helper("os_env", Box::new(contrib::handlebars::os_env_helper));
        handlebars
    };
}

#[derive(Deserialize, Default, Debug)]
pub struct SandboxPolicy {
    uid: Option<u32>,
    gid: Option<u32>,
    hostname: Option<String>,
    #[serde(default)]
    limits: Limits,
    #[serde(default)]
    mounts: Vec<Mount>,
    #[serde(default, rename = "env")]
    envs: HashMap<String, String>,
}

impl SandboxPolicy {
    pub fn from_str(data: &str) -> Result<Self> {
        let data = SANDBOX_POLICY_HANDLEBARS.render_template(data, &())?;
        let policy: Self = toml::from_str(&data)?;
        Ok(policy)
    }

    #[allow(non_snake_case)]
    pub(crate) fn KISS_POLICY() -> Self {
        let f = Embed::get("KISS-policy.toml").unwrap();
        let data = str::from_utf8(&f.data).unwrap();
        SandboxPolicy::from_str(data).unwrap()
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

        if let Some(id) = self.policy.uid {
            executor.uid(id);
        }
        if let Some(id) = self.policy.gid {
            executor.gid(id);
        }
        if let Some(hostname) = &self.policy.hostname {
            executor.hostname(hostname);
        }

        executor
            .limits(self.policy.limits.clone())
            .namespaces(Namespaces::default())
            .mounts(self.policy.mounts.clone());

        for (k, v) in self.policy.envs.iter() {
            executor.setenv(k, v);
        }

        executor
    }
}
