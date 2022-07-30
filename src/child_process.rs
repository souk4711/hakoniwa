use nix::unistd::execv;
use std::ffi::CString;

use crate::executor::Executor;

pub fn run(executor: &Executor) {
    exec(&executor.prog, &executor.argv);
}

fn exec<T: AsRef<str>>(prog: &str, argv: &[T]) {
    let prog = CString::new(prog).unwrap();
    let argv: Vec<_> = argv
        .iter()
        .map(|arg| CString::new(arg.as_ref()).unwrap())
        .collect();
    _ = execv(&prog, &argv);
}
