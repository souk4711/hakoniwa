use std::rc::Rc;

use crate::Command;

#[derive(Default)]
pub(crate) struct ContainerInner {}

#[derive(Default)]
pub struct Container {
    inner: Rc<ContainerInner>,
}

impl Container {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn command(&self, program: &str) -> Command {
        Command::new(program, Rc::clone(&self.inner))
    }
}
