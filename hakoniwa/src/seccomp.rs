use libseccomp::ScmpAction;
use serde::Deserialize;

/// Action to be taken when a filter rule dismatch/match.
#[derive(Deserialize, Clone, Copy, Default, Debug)]
#[serde(deny_unknown_fields)]
pub enum SeccompAction {
    /// This value results in immediate termination of the process,
    /// with a core dump. The system call is not executed.
    #[default]
    #[serde(rename = "kill_process")]
    KillProcess,

    /// This value results in the system call being executed after
    /// the filter return action is logged.
    #[serde(rename = "log")]
    Log,
}

impl SeccompAction {
    fn to_scmp_action(self) -> ScmpAction {
        match self {
            Self::KillProcess => ScmpAction::KillProcess,
            Self::Log => ScmpAction::Log,
        }
    }
}

#[derive(Deserialize, Default, Debug)]
#[serde(deny_unknown_fields)]
pub struct Seccomp {
    pub(crate) syscalls: Vec<String>,
    #[serde(default)]
    pub(crate) dismatch_action: SeccompAction,
}

impl Seccomp {
    pub fn new(dismatch_action: SeccompAction) -> Self {
        Self {
            dismatch_action,
            ..Default::default()
        }
    }

    pub fn dismatch_action(&self) -> ScmpAction {
        self.dismatch_action.to_scmp_action()
    }
}
