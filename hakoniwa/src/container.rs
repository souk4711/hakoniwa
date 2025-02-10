use std::rc::Rc;

use crate::Command;

pub(crate) struct ContainerInner {}

pub struct Container {
    inner: Rc<ContainerInner>,
}

impl Container {
    pub fn new() -> Self {
        Self {
            inner: Rc::new(ContainerInner {}),
        }
    }

    pub fn command(&self, program: &str) -> Command {
        Command::new(program, Rc::clone(&self.inner))
    }
}
