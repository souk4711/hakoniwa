use libseccomp::ScmpAction;
use serde::Deserialize;

#[derive(Deserialize, Default, Debug)]
pub struct Seccomp {
    pub(crate) syscalls: Vec<String>,
}

impl Seccomp {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn dismatch_action(&self) -> ScmpAction {
        ScmpAction::KillProcess
    }

    pub fn match_action(&self) -> ScmpAction {
        ScmpAction::Allow
    }
}
