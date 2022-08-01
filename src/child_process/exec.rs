use nix::{unistd, Result};
use std::ffi::CString;

pub fn exec<T: AsRef<str>>(prog: &str, argv: &[T]) -> Result<()> {
    let prog = CString::new(prog).unwrap();
    let argv: Vec<_> = argv
        .iter()
        .map(|arg| CString::new(arg.as_ref()).unwrap())
        .collect();
    unistd::execv(&prog, &argv)?;
    Ok(())
}
