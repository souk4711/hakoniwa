use std::{collections::HashMap, ffi::CString};

use crate::child_process::{error::Result, syscall};

pub(crate) fn exec<SA: AsRef<str>>(
    prog: &str,
    argv: &[SA],
    envp: &HashMap<String, String>,
) -> Result<()> {
    let prog = CString::new(prog).unwrap();
    let argv: Vec<_> = argv
        .iter()
        .map(|arg| CString::new(arg.as_ref()).unwrap())
        .collect();
    let envp: Vec<_> = envp
        .iter()
        .map(|(k, v)| CString::new(format!("{}={}", k, v)).unwrap())
        .collect();
    syscall::execve(&prog, &argv, &envp)
}
