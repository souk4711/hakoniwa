# Hakoniwa

## Usage

```rust
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
    container.rootfs("/").devfsmount("/dev").tmpfsmount("/tmp");

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
```

### More Examples

- [Unshare Namespace](./examples/container-unshare-namespace.rs)
- [Customized Mount](./examples/container-customized-mount.rs)
- [Network](./examples/container-network.rs)
- [Resource Limit](./examples/container-resource-limit.rs)
- [Landlock](./examples/container-landlock.rs)
- [Seccomp](./examples/container-seccomp.rs)
- [Handling IO](./examples/command-handling-io.rs)
- [Metric](./examples/command-metric.rs)
- [docs.rs](https://docs.rs/hakoniwa)
