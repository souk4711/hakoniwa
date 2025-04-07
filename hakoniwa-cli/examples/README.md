# Examples

## Usage

```sh
hakoniwa run -v \
  --unshare-all \
  --rootfs / --devfs /dev --tmpfs /tmp \
  -- dd if=/dev/random of=/tmp/output.txt count=1 bs=4
```

- `hakoniwa run -v`
  - **Run a COMMAND in a container**. By default, it will
  - Create a new `MOUNT` namespace
  - Create a new `USER` namespace and map current user to itself
  - Create a new `PID` namespace and mount a new `procfs` on `/proc`
  - With `-v` to show logging output
- `--unshare-all`
  - Create new `CGROUP`, `IPC`, `NETWORK`, `UTS`, ... namespaces
- `--rootfs / --devfs /dev --tmpfs /tmp`
  - Bind mount `/bin`, `/lib`, `/etc`, `/usr`, ... with read-only access if exists
  - Mount `devfs` on `/dev`, it contains a minimal set of device files, like `/dev/null`
  - Mount `tmpfs` on `/tmp`
- `-- dd if=/dev/random of=/tmp/output.txt count=1 bs=4`
  - Exec COMMAND `dd if=/dev/random of=/tmp/output.txt count=1 bs=4`

In most cases, you can just use following code (`--rootfs=/` is enabled by default):

```sh
hakoniwa run --unshare-all --devfs /dev --tmpfs /tmp -- COMMAND
```

For static linked binaries, it is not necassary to mount system-wide directories, use `--rootfs=none`:

```sh
hakoniwa run --unshare-all --rootfs=none --devfs /dev --tmpfs /tmp -b /mybin -- /mybin/static-linked-binaries-COMMAND
```

If you want access network, run with `--network pasta`:

```sh
hakoniwa run --unshare-all --devfs /dev --tmpfs /tmp --network=pasta -- wget https://example.com --spider
```

By default, it always loads a Podman-compatible seccomp profile, you can disable it using `--seccomp=unconfined`. Also you
can use `--limit-xxxx` to restrict process resource usage:

```sh
hakoniwa run --unshare-all --devfs /dev --tmpfs /tmp --limit-walltime 1 -- sleep 2
```

For debugging purpose, you can use `-v` or `-vv` to display the logging output.

```sh
hakoniwa run -v ...
```

If the command line is too long, too complex, you can [create a profile](./howto-introduction-to-config-file.md) and run with `--config`:

```sh
hakoniwa run -c myprofile.toml
```

### Command Reference

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
