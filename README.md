# Hakoniwa

Process isolation for Linux using namespaces, resource limits, landlock and seccomp.
It works by creating a new, completely empty, mount namespace where the root is
on a tmpdir, and will be automatically cleaned up when the last process exits.

It uses the following techniques:

- **Linux namespaces:** Create an isolated environment for the process.
- **MNT namespace + pivot_root:** Create a new root file system for the process.
- **NETWORK namespace + pasta**: Create a new user-mode networking stack for the process.
- **setrlimit:** Limit the amount of resources that can be used by the process.
- **landlock:** Restrict ambient rights (e.g. global filesystem access) for the process.
- **seccomp:** Restrict the system calls that the process can make.

It can help you with:

- Compile source code in a restricted sandbox, e.g. [makepkg with Hakoniwa][app-makepkg]
- Run browsers, or proprietary softwares in an isolated environment, e.g. [Firefox with Hakoniwa][app-firefox]

It also provides a set of profiles for the desktop application, read [Hakoniwa.d][hakoniwa.d] to learn more.

> [!WARNING]
> Running untrusted code is never safe, sandboxing cannot change this.

## Installation

### Pre-compiled binary

1. Install dependencies:

   - [libseccomp](https://github.com/libseccomp-rs/libseccomp-rs#requirements)
   - [passt](https://passt.top/passt/about/)

2. Download a pre-compiled binary from [Releases](https://github.com/souk4711/hakoniwa/releases).

3. Configure [AppArmor][troubleshooting-apparmor] or SELinux, if enabled.

### From source

1. Install dependencies:

   - [libseccomp](https://github.com/libseccomp-rs/libseccomp-rs#requirements)
   - [passt](https://passt.top/passt/about/)

2. Compile binary from source code:

   ```sh
   cargo install hakoniwa-cli --git https://github.com/souk4711/hakoniwa.git --locked
   ```

3. Configure [AppArmor][troubleshooting-apparmor] or SELinux, if enabled.

### Distros

#### Arch

```sh
sudo pacman -S libseccomp passt cargo

cargo install hakoniwa-cli --root ~/.cargo --locked
sudo mv ~/.cargo/bin/hakoniwa /usr/bin/hakoniwa
```

#### Fedora 41

```sh
sudo dnf install libseccomp-devel passt cargo

cargo install hakoniwa-cli --root ~/.cargo --locked
sudo mv ~/.cargo/bin/hakoniwa /usr/bin/hakoniwa

sudo dnf install container-selinux
sudo chcon -u system_u -t container_runtime_exec_t /usr/bin/hakoniwa
```

#### Ubuntu 24.04

```sh
sudo apt install libseccomp-dev passt cargo

cargo install hakoniwa-cli --root ~/.cargo --locked
sudo mv ~/.cargo/bin/hakoniwa /usr/bin/hakoniwa

curl -o apparmor.d-hakoniwa https://raw.githubusercontent.com/souk4711/hakoniwa/refs/heads/main/etc/apparmor.d/hakoniwa
sudo mv apparmor.d-hakoniwa /etc/apparmor.d/hakoniwa
sudo systemctl reload apparmor.service
```

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

$ hakoniwa run -v --config /etc/hakoniwa.d/firefox.toml
[2025-04-27T07:01:41Z DEBUG] CONFIG: /etc/hakoniwa.d/firefox.toml
[2025-04-27T07:01:41Z DEBUG] CONFIG: Including /etc/hakoniwa.d/abstractions/os/linux.toml
[2025-04-27T07:01:41Z DEBUG] CONFIG: Including /etc/hakoniwa.d/abstractions/device/dri.toml
[2025-04-27T07:01:41Z DEBUG] CONFIG: Including /etc/hakoniwa.d/abstractions/device/sound.toml
[2025-04-27T07:01:41Z DEBUG] CONFIG: Including /etc/hakoniwa.d/abstractions/socket/dbus-session.toml
[2025-04-27T07:01:41Z DEBUG] CONFIG: Including /etc/hakoniwa.d/abstractions/socket/dbus-system.toml
[2025-04-27T07:01:41Z DEBUG] CONFIG: Including /etc/hakoniwa.d/abstractions/socket/pipewire.toml
[2025-04-27T07:01:41Z DEBUG] CONFIG: Including /etc/hakoniwa.d/abstractions/socket/pulseaudio.toml
[2025-04-27T07:01:41Z DEBUG] CONFIG: Including /etc/hakoniwa.d/abstractions/socket/wayland.toml
[2025-04-27T07:01:41Z DEBUG] CONFIG: Including /etc/hakoniwa.d/abstractions/socket/x11.toml
[2025-04-27T07:01:41Z DEBUG] CONFIG: Including /etc/hakoniwa.d/abstractions/network/mode/pasta.toml
[2025-04-27T07:01:41Z DEBUG] CONFIG: Including /etc/hakoniwa.d/abstractions/network/connect/http.toml
[2025-04-27T07:01:41Z DEBUG] CONFIG: Including /etc/hakoniwa.d/abstractions/network/connect/https.toml
[2025-04-27T07:01:41Z DEBUG] Unshare namespaces: CloneFlags(CLONE_NEWNS | CLONE_NEWCGROUP | CLONE_NEWUTS | CLONE_NEWIPC | CLONE_NEWUSER | CLONE_NEWPID | CLONE_NEWNET)
[2025-04-27T07:01:41Z DEBUG] RootDir: "/tmp/hakoniwa-mylPUJ" -> "/"
...
[2025-04-27T07:01:41Z DEBUG] Execve: "/usr/bin/firefox", []
...
```

More examples can be found in [hakoniwa-cli](https://github.com/souk4711/hakoniwa/tree/main/hakoniwa-cli).

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
        // .landlock_ruleset(..)   // Set landlock ruleset
        // .seccomp_filter(..)     // Set seccomp filter
        .command("/bin/sh")     // Create Command
        .status()               // Execute
        .expect("failed to execute process witnin container");
}
```

More examples can be found in [hakoniwa](https://github.com/souk4711/hakoniwa/tree/main/hakoniwa).

## Implementation of Command::status

![Implementation of Command::staus]

## Acknowledgements

- Special thanks to [bubblewrap](https://github.com/containers/bubblewrap).

## License

The CLI is licensed under the [GPL-3.0-only].

The Library is licensed under the [LGPL-3.0 WITH LGPL-3.0-linking-exception].

[hakoniwa.d]: https://github.com/souk4711/hakoniwa.d
[app-firefox]: https://github.com/souk4711/hakoniwa/tree/main/hakoniwa-cli/docs/app-firefox
[app-makepkg]: https://github.com/souk4711/hakoniwa/tree/main/hakoniwa-cli/docs/app-makepkg
[troubleshooting-apparmor]: https://github.com/souk4711/hakoniwa/blob/main/hakoniwa-cli/docs/troubleshooting-apparmor
[Implementation of Command::staus]: https://github.com/souk4711/hakoniwa/raw/main/architecture.svg
[GPL-3.0-only]: https://github.com/souk4711/hakoniwa/blob/main/hakoniwa-cli/LICENSE
[LGPL-3.0 WITH LGPL-3.0-linking-exception]: https://github.com/souk4711/hakoniwa/blob/main/hakoniwa/LICENSE
