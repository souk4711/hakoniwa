use crate::executor::Executor;

pub struct Sandbox {}

impl Sandbox {
    pub fn new() -> Sandbox {
        Sandbox {}
    }

    pub fn command<T: AsRef<str>>(&self, prog: &str, argv: &[T]) -> Executor {
        Executor::new(prog, argv)
    }
}
