use hakoniwa::{Error, ExecutorResultStatus, Sandbox, SandboxPolicy};

fn main() -> Result<(), Error> {
    let mut sandbox = Sandbox::new();
    sandbox.with_policy(SandboxPolicy::from_str(
        r#"
mounts = [
  { source = "/bin"  , target = "/bin"  },
  { source = "/lib"  , target = "/lib"  },
  { source = "/lib64", target = "/lib64"},
  { source = "/usr"  , target = "/usr"  },
]
    "#,
    )?);

    // Enabled with necessary x86_64 syscalls.
    let prog = "echo";
    let argv = vec![prog, "Hako!"];
    let mut executor = sandbox.command(prog, &argv);
    let result = executor
        .seccomp_enable()
        .seccomp_allow("access")?
        .seccomp_allow("arch_prctl")?
        .seccomp_allow("brk")?
        .seccomp_allow("close")?
        .seccomp_allow("execve")? // this syscall is always required
        .seccomp_allow("exit_group")?
        .seccomp_allow("getrandom")?
        .seccomp_allow("mmap")?
        .seccomp_allow("mprotect")?
        .seccomp_allow("munmap")?
        .seccomp_allow("newfstatat")?
        .seccomp_allow("openat")?
        .seccomp_allow("pread64")?
        .seccomp_allow("prlimit64")?
        .seccomp_allow("read")?
        .seccomp_allow("rseq")?
        .seccomp_allow("set_robust_list")?
        .seccomp_allow("set_tid_address")?
        .seccomp_allow("write")?
        .run();
    assert_eq!(result.status, ExecutorResultStatus::Ok);
    assert_eq!(result.exit_code, Some(0));
    assert_eq!(String::from_utf8_lossy(&result.stdout), "Hako!\n");

    Ok(())
}

#[test]
fn test_main() {
    main().unwrap();
}
