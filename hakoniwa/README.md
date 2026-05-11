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
        ruleset.allow_path("/bin", FsAccess::R | FsAccess::X);
        ruleset.allow_path("/lib", FsAccess::R | FsAccess::X);
        #[cfg(target_arch = "x86_64")]
        ruleset.allow_path("/lib64", FsAccess::R | FsAccess::X);
        ruleset.allow_path("/usr", FsAccess::R);
        ruleset.allow_path("/dev", FsAccess::R);
        ruleset.allow_path("/tmp", FsAccess::W);
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
- [Network - Pasta](./examples/container-network-pasta.rs)
- [Network - RustSlirp](./examples/container-network-rustslirp.rs)
- [Resource Limit](./examples/container-resource-limit.rs)
- [Control Groups](./examples/container-cgroups.rs)
- [Landlock](./examples/container-landlock.rs)
- [Seccomp](./examples/container-seccomp.rs)
- [Command From Closure](./examples/command-from-closure.rs)
- [Command Handling IO](./examples/command-handling-io.rs)
- [Command Metric](./examples/command-metric.rs)
- [docs.rs](https://docs.rs/hakoniwa)

## Troubleshooting

If you receive `hakoniwa: ... => Operation not permitted (os error 1)`, read following docs:

- [Permission denied](./docs/troubleshooting-eperm)
