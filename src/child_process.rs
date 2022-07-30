use nix::unistd;
use std::ffi::CString;

use crate::executor::Executor;
use crate::limits::Limits;

pub fn run(executor: &Executor) {
    set_rlimits(&executor.limits);

    _ = unistd::chdir(&executor.dir);
    exec(&executor.prog, &executor.argv);
}

fn set_rlimits(limits: &Limits) {
    set_rlimit(libc::RLIMIT_AS, limits.r#as);
    set_rlimit(libc::RLIMIT_CPU, limits.cpu);
    set_rlimit(libc::RLIMIT_CORE, limits.core);
    set_rlimit(libc::RLIMIT_FSIZE, limits.fsize);
    set_rlimit(libc::RLIMIT_NOFILE, limits.nofile);
}

fn set_rlimit(resource: libc::__rlimit_resource_t, limit: Option<u64>) {
    if let Some(limit) = limit {
        let rlimit = libc::rlimit {
            rlim_cur: limit,
            rlim_max: limit,
        };
        unsafe {
            libc::setrlimit(resource, &rlimit);
        }
    }
}

fn exec<T: AsRef<str>>(prog: &str, argv: &[T]) {
    let prog = CString::new(prog).unwrap();
    let argv: Vec<_> = argv
        .iter()
        .map(|arg| CString::new(arg.as_ref()).unwrap())
        .collect();
    _ = unistd::execv(&prog, &argv);
}
