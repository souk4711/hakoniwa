# Hakoniwa

Process isolation for Linux using namespaces, resource limits and seccomp. It
works by creating a new, completely empty, mount namespace where the root is
on a tmpfs that is invisible from the host, and will be automatically cleaned
up when the last process exits.

## Installation

### Cargo

- Install libseccomp by following [this](https://github.com/libseccomp-rs/libseccomp-rs#requirements) guide.
- Install the rust toolchain in order to have cargo installed by following [this](https://www.rust-lang.org/tools/install) guide.
- Run `cargo install hakoniwa-cli`.

## Usage

### CLI

```console
$ hakoniwa run -- /bin/bash
bash: cannot set terminal process group (-1): Inappropriate ioctl for device
bash: no job control in this shell
bash-5.2$ pwd
/
bash-5.2$ ls
bin  etc  lib  lib64  proc  sbin  usr
bash-5.2$ ls /proc
1           bus        crypto         execdomains  ioports    kmsg         meminfo  net           self      sysrq-trigger  version
3           cgroups    devices        fb           irq        kpagecgroup  misc     pagetypeinfo  slabinfo  sysvipc        vmallocinfo
acpi        cmdline    diskstats      filesystems  kallsyms   kpagecount   modules  partitions    softirqs  thread-self    vmstat
asound      config.gz  dma            fs           kcore      kpageflags   mounts   pressure      stat      timer_list     zoneinfo
bootconfig  consoles   driver         interrupts   keys       loadavg      mtd      schedstat     swaps     tty
buddyinfo   cpuinfo    dynamic_debug  iomem        key-users  locks        mtrr     scsi          sys       uptime
bash-5.2$
```

More examples can be found in [hakoniwa-cli/examples](./hakoniwa-cli/examples).

### Rust Library

The code below is almost eq to `hakoniwa run -- /bin/bash`:

```rust
use hakoniwa::Container;

fn main() {
    _ = Container::new()        // Create Container with new namespaces via unshare
        .rootfs("/")            // Mount necessary directories, e.g. `/bin`
        .command("/bin/bash")   // Create Command
        .status()               // Execute
        .expect("failed to execute process witnin container");
}
```

More examples can be found in [hakoniwa/examples](./hakoniwa/examples).

## Acknowledgements

- Special thanks to [bubblewrap](https://github.com/containers/bubblewrap).

## License

The CLI is licensed under the [GPL-3.0-only](./hakoniwa-cli/LICENSE).

The Library is licensed under the [LGPL-3.0-linking-exception](./hakoniwa/LICENSE).
