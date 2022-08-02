use nix::unistd;
use std::ffi::CString;

use crate::{tryfn, Result};

pub fn exec<T: AsRef<str>>(prog: &str, argv: &[T]) -> Result<()> {
    let prog = CString::new(prog).unwrap();
    let argv: Vec<_> = argv
        .iter()
        .map(|arg| CString::new(arg.as_ref()).unwrap())
        .collect();
    let env: [CString; 0] = [];

    tryfn!(
        unistd::execve(&prog, &argv, &env),
        "execve({:?}, ...)",
        prog
    )?;
    Ok(())
}
