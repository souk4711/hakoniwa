#[cfg(feature = "seccomp")]
fn main() -> Result<(), hakoniwa::Error> {
    use hakoniwa::{scmp_argcmp, seccomp::*};

    let mut filter = Filter::new(Action::Errno(libc::EPERM));

    #[cfg(target_arch = "x86_64")]
    {
        filter.add_arch(Arch::X8664);
        filter.add_arch(Arch::X86);
        filter.add_arch(Arch::X32);
    }

    filter.add_rule(Action::Allow, "access");
    filter.add_rule(Action::Allow, "arch_prctl");
    filter.add_rule(Action::Allow, "brk");
    filter.add_rule(Action::Allow, "close");
    filter.add_rule(Action::Allow, "execve");
    filter.add_rule(Action::Allow, "exit_group");
    filter.add_rule(Action::Allow, "fstat");
    filter.add_rule(Action::Allow, "getrandom");
    filter.add_rule(Action::Allow, "mmap");
    filter.add_rule(Action::Allow, "mprotect");
    filter.add_rule(Action::Allow, "munmap");
    filter.add_rule(Action::Allow, "newfstatat");
    filter.add_rule(Action::Allow, "openat");
    filter.add_rule(Action::Allow, "pread64");
    filter.add_rule(Action::Allow, "prlimit64");
    filter.add_rule(Action::Allow, "read");
    filter.add_rule(Action::Allow, "rseq");
    filter.add_rule(Action::Allow, "set_robust_list");
    filter.add_rule(Action::Allow, "set_tid_address");
    filter.add_rule(Action::Allow, "stat");
    filter.add_rule(Action::Allow, "write");
    filter.add_rule_conditional(Action::Allow, "personality", &[scmp_argcmp!(arg0 == 0)]);
    filter.add_rule_conditional(Action::Allow, "personality", &[scmp_argcmp!(arg0 == 8)]);

    let mut container = hakoniwa::Container::new();
    container.rootfs("/").seccomp_filter(filter);

    let status = container.command("/bin/echo").status()?;
    assert!(status.success());

    Ok(())
}

#[cfg(not(feature = "seccomp"))]
fn main() -> Result<(), hakoniwa::Error> {
    Ok(())
}

#[test]
fn test_main() {
    main().unwrap();
}
