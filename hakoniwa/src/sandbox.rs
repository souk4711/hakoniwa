use handlebars::Handlebars;
use lazy_static::lazy_static;
use serde::Deserialize;
use std::{collections::HashMap, str};

use crate::{
    contrib::handlebars::{fs_read_to_string_helper, os_env_helper, os_homedir_helper},
    error, Executor, File, Limits, Mount, Result, Seccomp,
};

lazy_static! {
    static ref SANDBOX_POLICY_HANDLEBARS: Handlebars<'static> = {
        let mut handlebars = Handlebars::new();
        handlebars.register_helper("fs_read_to_string", Box::new(fs_read_to_string_helper));
        handlebars.register_helper("os_env", Box::new(os_env_helper));
        handlebars.register_helper("os_homedir", Box::new(os_homedir_helper));
        handlebars
    };
}

/// Sandbox policy configuration use TOML format.
#[derive(Deserialize, Default, Debug)]
#[serde(deny_unknown_fields)]
pub struct SandboxPolicy {
    share_net: Option<bool>,
    share_uts: Option<bool>,
    uid: Option<u32>,
    gid: Option<u32>,
    hostname: Option<String>,
    #[serde(default)]
    mounts: Vec<Mount>,
    #[serde(default)]
    files: Vec<File>,
    #[serde(default)]
    env: HashMap<String, String>,
    #[serde(default)]
    limits: Limits,
    #[serde(default)]
    seccomp: Option<Seccomp>,
}

impl SandboxPolicy {
    /// Create a policy from a string.
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(data: &str) -> Result<Self> {
        let data = SANDBOX_POLICY_HANDLEBARS
            .render_template(data, &())
            .map_err(error::ParseConfigurationErrorKind::HandlebarsRenderError)?;
        let policy: Self =
            toml::from_str(&data).map_err(error::ParseConfigurationErrorKind::TomlError)?;
        Ok(policy)
    }
}

/// Create [Executor](super::Executor) with a shared policy configuration.
#[derive(Default)]
pub struct Sandbox {
    policy: Option<SandboxPolicy>,
}

impl Sandbox {
    /// Constructor.
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    /// Use a specified policy configuration, will used in [command()](Self::command()).
    pub fn with_policy(&mut self, policy: SandboxPolicy) -> &mut Self {
        self.policy = Some(policy);
        self
    }

    /// Create a [Executor](super::Executor).
    pub fn command<SA: AsRef<str>>(&self, prog: &str, argv: &[SA]) -> Executor {
        let mut executor = Executor::new(prog, argv);
        let policy = match &self.policy {
            Some(val) => val,
            None => return executor,
        };

        if let Some(share) = policy.share_net {
            executor.share_net_ns(share);
        }
        if let Some(share) = policy.share_uts {
            executor.share_uts_ns(share);
        }

        if let Some(id) = policy.uid {
            executor.uid(id);
        }
        if let Some(id) = policy.gid {
            executor.gid(id);
        }
        if let Some(hostname) = &policy.hostname {
            executor.hostname(hostname);
        }

        executor.mounts(&policy.mounts);
        executor.files(&policy.files);

        for (k, v) in policy.env.iter() {
            executor.setenv(k, v);
        }

        executor.limits(&policy.limits);
        executor.seccomp(&policy.seccomp);
        executor
    }
}
