use libseccomp::ScmpAction;
use serde::Deserialize;

#[derive(Deserialize, Default, Debug)]
pub enum SeccompAction {
    #[default]
    KillProcess,
    Allow,
}

impl SeccompAction {
    fn to_scmp_action(&self) -> ScmpAction {
        match self {
            Self::KillProcess => ScmpAction::KillProcess,
            Self::Allow => ScmpAction::Allow,
        }
    }
}

#[derive(Deserialize, Default, Debug)]
pub struct Seccomp {
    pub(crate) syscalls: Vec<String>,
    #[serde(skip)]
    pub(crate) dismatch_action: SeccompAction,
    #[serde(skip)]
    pub(crate) match_action: SeccompAction,
}

impl Seccomp {
    pub fn new() -> Self {
        Self {
            dismatch_action: SeccompAction::KillProcess,
            match_action: SeccompAction::Allow,
            ..Default::default()
        }
    }

    pub fn dismatch_action(&self) -> ScmpAction {
        self.dismatch_action.to_scmp_action()
    }

    pub fn match_action(&self) -> ScmpAction {
        self.match_action.to_scmp_action()
    }
}
