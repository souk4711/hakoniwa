use serde::Deserialize;

#[derive(Deserialize, Default, Debug)]
pub struct Seccomp {
    syscalls: Vec<String>,
}

impl Seccomp {
    pub fn is_enabled(&self) -> bool {
        self.syscalls.len() > 0
    }
}
