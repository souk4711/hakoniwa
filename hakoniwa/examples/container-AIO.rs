use hakoniwa::*;

fn main() -> Result<()> {
    // unshare User, Mount, PID namespaces
    let mut container = Container::new();

    // unshare Cgroup, IPC, Network, UTS namespaces
    container
        .unshare(Namespace::Cgroup)
        .unshare(Namespace::Ipc)
        .unshare(Namespace::Network)
        .unshare(Namespace::Uts);

    // mount filesystem
    container.rootfs("/")?.devfsmount("/dev").tmpfsmount("/tmp");

    // optional: network
    let pasta = Pasta::default();
    container.network(pasta);

    // optional: rlimit
    container
        .setrlimit(Rlimit::Core, 0, 0) // no core file
        .setrlimit(Rlimit::Nofile, 32, 32); // 32 max fd

    // optional: landlock
    #[cfg(feature = "landlock")]
    {
        use hakoniwa::landlock::*;
        let mut ruleset = Ruleset::default();
        ruleset.restrict(Resource::FS, CompatMode::Enforce);
        ruleset.add_fs_rule("/bin", FsAccess::R | FsAccess::X);
        ruleset.add_fs_rule("/lib", FsAccess::R | FsAccess::X);
        #[cfg(target_arch = "x86_64")]
        ruleset.add_fs_rule("/lib64", FsAccess::R | FsAccess::X);
        ruleset.add_fs_rule("/usr", FsAccess::R);
        ruleset.add_fs_rule("/dev", FsAccess::R);
        ruleset.add_fs_rule("/tmp", FsAccess::W);
        container.landlock_ruleset(ruleset);
    }

    // optional: seccomp
    #[cfg(feature = "seccomp")]
    {
        use hakoniwa::seccomp::*;
        let filter = Filter::new(Action::Log);
        container.seccomp_filter(filter);
    }

    // create command
    let mut command = container.command("/bin/dd");
    command.args(["if=/dev/random", "of=/tmp/output.txt", "count=1", "bs=4"]);

    // run
    let status = command.status()?;
    assert!(status.success());
    Ok(())
}

#[test]
fn test_main() {
    main().unwrap();
}
