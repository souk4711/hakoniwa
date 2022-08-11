use serde::Deserialize;

#[derive(Deserialize, Default, Debug)]
pub struct Seccomp {
    pub(crate) syscalls: Vec<String>,
}
