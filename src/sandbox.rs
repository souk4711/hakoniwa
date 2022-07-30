use crate::executor::{Executor, ExecutorResult};

pub struct Sandbox {}

impl Sandbox {
    pub fn new() -> Sandbox {
        Sandbox {}
    }

    pub fn run<T: AsRef<str>>(&self, prog: &str, argv: &[T]) -> ExecutorResult {
        let mut executor = Executor::new(prog, argv);
        executor.run()
    }
}
