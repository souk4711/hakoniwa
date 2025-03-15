# Hakoniwa

Process isolation for Linux using namespaces, resource limits and seccomp. It
works by creating a new, completely empty, mount namespace where the root is
on a tmpdir, and will be automatically cleaned up when the last process exits.

It uses the following techniques:

- **Linux namespaces:** Create an isolated environment for the process.
- **MNT namespace + pivot_root:** Create a new root file system for the process.
- **setrlimit:** Limit the amount of resources that can be used by the process.
- **seccomp:** Limit the set of system calls that can be used by the process.

## Installation

### Cargo

- Install libseccomp by following [this](https://github.com/libseccomp-rs/libseccomp-rs#requirements) guide.
- Install the rust toolchain in order to have cargo installed by following [this](https://www.rust-lang.org/tools/install) guide.
- Run `cargo install hakoniwa-cli`.

## Usage

### CLI

```console
$ hakoniwa run -- sh
sh-5.2$ pwd
/
sh-5.2$ ls
bin  etc  lib  lib64  proc  sbin  usr
sh-5.2$ ls /proc
1           bus        crypto         execdomains  ioports    kmsg         meminfo  net           self      sysrq-trigger  version
3           cgroups    devices        fb           irq        kpagecgroup  misc     pagetypeinfo  slabinfo  sysvipc        vmallocinfo
acpi        cmdline    diskstats      filesystems  kallsyms   kpagecount   modules  partitions    softirqs  thread-self    vmstat
asound      config.gz  dma            fs           kcore      kpageflags   mounts   pressure      stat      timer_list     zoneinfo
bootconfig  consoles   driver         interrupts   key-users  loadavg      mtd      schedstat     swaps     tty
buddyinfo   cpuinfo    dynamic_debug  iomem        keys       locks        mtrr     scsi          sys       uptime
sh-5.2$ ps aux
USER         PID %CPU %MEM    VSZ   RSS TTY      STAT START   TIME COMMAND
johndoe        1  0.0  0.0   4708  4020 ?        S    21:22   0:00 /usr/bin/sh
johndoe        4  0.0  0.0   6620  3896 ?        R+   21:22   0:00 ps aux
sh-5.2$ exit
exit
```

More examples can be found in [hakoniwa-cli/examples].

### Rust Library

The code below is almost eq to `hakoniwa run -- sh`:

```rust
use hakoniwa::Container;

fn main() {
    _ = Container::new()        // Create Container with new namespaces via unshare
        .rootfs("/")            // Mount necessary directories, e.g. `/bin`
        // .devfsmount("/dev")     // Mount `devfs` on `/dev`, it contains a minimal set of device files, like `/dev/null`
        // .tmpfsmount("/tmp")     // Mount `tmpfs` on `/tmp`
        // .setrlimit(..)          // Set resource limits
        .command("/bin/sh")     // Create Command
        .status()               // Execute
        .expect("failed to execute process witnin container");
}
```

More examples can be found in [hakoniwa/examples].

## Implementation of Command::status

![Implementation of Command::staus]

## Acknowledgements

- Special thanks to [bubblewrap](https://github.com/containers/bubblewrap).

## License

The CLI is licensed under the [GPL-3.0-only].

The Library is licensed under the [LGPL-3.0 WITH LGPL-3.0-linking-exception].

[hakoniwa-cli/examples]: https://github.com/souk4711/hakoniwa/tree/main/hakoniwa-cli/examples
[hakoniwa/examples]: https://github.com/souk4711/hakoniwa/tree/main/hakoniwa/examples
[GPL-3.0-only]: https://github.com/souk4711/hakoniwa/blob/main/hakoniwa-cli/LICENSE
[LGPL-3.0 WITH LGPL-3.0-linking-exception]: https://github.com/souk4711/hakoniwa/blob/main/hakoniwa/LICENSE
[Implementation of Command::staus]: https://github.com/souk4711/hakoniwa/raw/main/docs/implementation-of-runc.svg
