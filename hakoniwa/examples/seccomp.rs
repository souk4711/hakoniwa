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
        .seccomp_syscall_add("access")?
        .seccomp_syscall_add("arch_prctl")?
        .seccomp_syscall_add("brk")?
        .seccomp_syscall_add("close")?
        .seccomp_syscall_add("execve")?
        .seccomp_syscall_add("exit_group")?
        .seccomp_syscall_add("fstat")?
        .seccomp_syscall_add("getrandom")?
        .seccomp_syscall_add("mmap")?
        .seccomp_syscall_add("mprotect")?
        .seccomp_syscall_add("munmap")?
        .seccomp_syscall_add("newfstatat")?
        .seccomp_syscall_add("openat")?
        .seccomp_syscall_add("pread64")?
        .seccomp_syscall_add("prlimit64")?
        .seccomp_syscall_add("read")?
        .seccomp_syscall_add("rseq")?
        .seccomp_syscall_add("set_robust_list")?
        .seccomp_syscall_add("set_tid_address")?
        .seccomp_syscall_add("stat")?
        .seccomp_syscall_add("write")?
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
