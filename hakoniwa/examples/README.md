# Examples

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
    container
        .bindmount_ro("/bin", "/bin")
        .bindmount_ro("/lib", "/lib")
        .bindmount_ro("/lib64", "/lib64")
        .bindmount_ro("/usr", "/usr")
        .devfsmount("/dev")
        .tmpfsmount("/tmp");

    // resource limit
    container
        .setrlimit(Rlimit::As, 16_000_000, 16_000_000) // 16MB
        .setrlimit(Rlimit::Core, 0, 0) // no core file
        .setrlimit(Rlimit::Nofile, 32, 32); // 32 max fd

    // create command
    let mut command = container.command("/bin/dd");
    command.args(["if=/dev/random", "of=/tmp/output.txt", "count=1", "bs=4"]);

    // run
    let status = command.status()?;
    assert!(status.success());
    Ok(())
}
```

### Document

- [Unshare Namespace](./container-unshare-namespace.rs)
- [Customized Mount](./container-customized-mount.rs)
- [Resource Limit](./container-resource-limit.rs)
- [Seccomp](./container-seccomp.rs)
- [Handling IO](./command-handling-io.rs)
- [Metric](./command-metric.rs)
