# Examples

## Usage

### Shell Explain

```sh
hakoniwa run --unshare-all --rootfs / --devfs /dev --tmpfs /tmp --limit-walltime 60 -- dd if=/dev/random of=/tmp/output.txt count=1 bs=4
```

- `hakoniwa run`
  - **Run a COMMAND in a container**. By default, it will
  - Create a new `MOUNT` namespace
  - Create a new `USER` namespace and map current user to itself
  - Create a new `PID` namespace and mount a new `procfs` on `/proc`
- `--unshare-all`
  - Create a new `CGROUP` namespace
  - Create a new `IPC` namespace
  - ..
- `--rootfs /`
  - Bind mount `/bin` on `/bin` with read-only access if exists
  - Bind mount `/lib` on `/lib` with read-only access if exists
  - ..
- `--devfs /dev`
  - Mount `devfs` on `/dev`, it contains a minimal set of device files, like `/dev/null`
- `--tmpfs /tmp`
  - Mount `tmpfs` on `/tmp`
- `--limit-walltime 60`
  - Limit the amount of wall time that the COMMAND can consume
- `-- dd if=/dev/random of=/tmp/output.txt count=1 bs=4`
  - Exec COMMAND `dd if=/dev/random of=/tmp/output.txt count=1 bs=4`

### Document

- [Unshare Linux Namespace](./usage-unshare.md)
- [Mount FileSystem](./usage-mount.md)
- [Process Resource Limit](./usage-limit.md)
- [Landlock](./usage-landlock.md)
- [Seccomp Profile](./usage-seccomp.md)
- [Network](./usage-network.md)
- [Misc](./usage-misc.md)
- [Config](./usage-config.md)
- [COMMAND](./usage-command.md)

## HowTo

- [Launch CLI App](./howto-launch-cli-app.md)
- [Launch Desktop App](./howto-launch-desktop-app.md)
- [Introduction to Config File](./howto-introduction-to-config-file.md)
- [Introduction to Seccomp Profile](./howto-introduction-to-seccomp-profile.md)
