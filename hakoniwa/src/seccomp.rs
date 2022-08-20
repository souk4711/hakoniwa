use libseccomp::ScmpAction;
use serde::Deserialize;

#[derive(Deserialize, Clone, Default, Debug)]
pub enum SeccompAction {
    #[default]
    #[serde(rename = "kill_process")]
    KillProcess,
    #[serde(rename = "allow")]
    Allow,
    #[serde(rename = "log")]
    Log,
}

impl SeccompAction {
    fn to_scmp_action(&self) -> ScmpAction {
        match self {
            Self::KillProcess => ScmpAction::KillProcess,
            Self::Allow => ScmpAction::Allow,
            Self::Log => ScmpAction::Log,
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct Seccomp {
    pub(crate) syscalls: Vec<String>,
    #[serde(default = "Seccomp::default_dismatch_action")]
    pub(crate) dismatch_action: SeccompAction,
    #[serde(default = "Seccomp::default_match_action")]
    pub(crate) match_action: SeccompAction,
}

impl Seccomp {
    pub fn new(dismatch_action: SeccompAction, match_action: SeccompAction) -> Self {
        Self {
            dismatch_action,
            match_action,
            syscalls: vec![],
        }
    }

    pub fn dismatch_action(&self) -> ScmpAction {
        self.dismatch_action.to_scmp_action()
    }

    pub fn match_action(&self) -> ScmpAction {
        self.match_action.to_scmp_action()
    }

    fn default_dismatch_action() -> SeccompAction {
        SeccompAction::KillProcess
    }

    fn default_match_action() -> SeccompAction {
        SeccompAction::Allow
    }
}

impl Default for Seccomp {
    fn default() -> Self {
        Self {
            dismatch_action: Self::default_dismatch_action(),
            match_action: Self::default_match_action(),
            syscalls: vec![],
        }
    }
}
