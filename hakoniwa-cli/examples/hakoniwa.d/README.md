# Hakoniwa.d

## File Structure

```console,ignore
.                               # HOME
├── .config/hakoniwa.d/         # Hakoniwa Configuration Files
│   └── abstractions/
│       ├── audio.toml          # Sound Card, Pulse Audio, etc.
│       ├── base.toml           # Base Sandbox Environment - unshare-all, rootfs, etc.
│       ├── dbus-session.toml   # DBus - session bus
│       ├── dbus-system.toml    # DBus - system bus
│       ├── graphics.toml       # DRI, etc.
│       ├── x11.toml            # X11
│       └── ...
│   ├── firefox.toml            # App - Firefox
│   ├── fish.toml               # App - Fish
│   └── ...
│
├── .local/share/hakoniwa/      # Hakoniwa App Data
│   └── apps/
│       ├── firefox/            # HOME for App Firefox
│       ├── fish/               # HOME for App Fish
│       └── ...
```

## Fish

```console,ignore
$ mkdir -p ~/.local/share/hakoniwa/apps/fish
$ hakoniwa run -v -c ~/.config/hakoniwa.d/fish.toml
[2025-03-28T18:24:39Z DEBUG] CONFIG: /home/johndoe/.config/hakoniwa.d/fish.toml
[2025-03-28T18:24:39Z DEBUG] CONFIG: Including /home/johndoe/.config/hakoniwa.d/abstractions/base.toml
[2025-03-28T18:24:39Z DEBUG] Unshare namespaces: CloneFlags(CLONE_NEWNS | CLONE_NEWCGROUP | CLONE_NEWUTS | CLONE_NEWIPC | CLONE_NEWUSER | CLONE_NEWPID | CLONE_NEWNET)
[2025-03-28T18:24:39Z DEBUG] RootDir: "/tmp/hakoniwa-ySsZZ9" -> "/"
[2025-03-28T18:24:39Z DEBUG] Mount: "/usr/bin" -> "/bin", FsType(), MsFlags(MS_RDONLY | MS_NOSUID | MS_BIND | MS_REC)
[2025-03-28T18:24:39Z DEBUG] Mount: "devfs" -> "/dev", FsType(devfs), MsFlags(0x0)
[2025-03-28T18:24:39Z DEBUG] Mount: "/etc" -> "/etc", FsType(), MsFlags(MS_RDONLY | MS_NOSUID | MS_BIND | MS_REC)
[2025-03-28T18:24:39Z DEBUG] Mount: "/home/johndoe/.local/share/hakoniwa/apps/fish" -> "/home/johndoe", FsType(), MsFlags(MS_NOSUID | MS_BIND | MS_REC)
[2025-03-28T18:24:39Z DEBUG] Mount: "/usr/lib" -> "/lib", FsType(), MsFlags(MS_RDONLY | MS_NOSUID | MS_BIND | MS_REC)
[2025-03-28T18:24:39Z DEBUG] Mount: "/usr/lib" -> "/lib64", FsType(), MsFlags(MS_RDONLY | MS_NOSUID | MS_BIND | MS_REC)
[2025-03-28T18:24:39Z DEBUG] Mount: "proc" -> "/proc", FsType(proc), MsFlags(MS_NOSUID | MS_NODEV | MS_NOEXEC)
[2025-03-28T18:24:39Z DEBUG] Mount: "tmpfs" -> "/run", FsType(tmpfs), MsFlags(MS_NOSUID | MS_NODEV)
[2025-03-28T18:24:39Z DEBUG] Mount: "/usr/bin" -> "/sbin", FsType(), MsFlags(MS_RDONLY | MS_NOSUID | MS_BIND | MS_REC)
[2025-03-28T18:24:39Z DEBUG] Mount: "tmpfs" -> "/sys", FsType(tmpfs), MsFlags(MS_NOSUID | MS_NODEV)
[2025-03-28T18:24:39Z DEBUG] Mount: "tmpfs" -> "/tmp", FsType(tmpfs), MsFlags(MS_NOSUID | MS_NODEV)
[2025-03-28T18:24:39Z DEBUG] Mount: "/usr" -> "/usr", FsType(), MsFlags(MS_RDONLY | MS_NOSUID | MS_BIND | MS_REC)
[2025-03-28T18:24:39Z DEBUG] UID mapping: container_id: 1000, host_id: 1000, count: 1
[2025-03-28T18:24:39Z DEBUG] GID mapping: container_id: 1000, host_id: 1000, count: 1
[2025-03-28T18:24:39Z DEBUG] Env: HOME=/home/johndoe
[2025-03-28T18:24:39Z DEBUG] Env: LANGUAGE=en_US
[2025-03-28T18:24:39Z DEBUG] Env: LANG=en_US.UTF-8
[2025-03-28T18:24:39Z DEBUG] Env: TERM=xterm-256color
[2025-03-28T18:24:39Z DEBUG] Landlock: Load 13 FS rules
[2025-03-28T18:24:39Z DEBUG] Seccomp: Load 439 rules for architectures(X8664, X86, X32)
[2025-03-28T18:24:39Z DEBUG] Execve: "/usr/bin/fish", []
[2025-03-28T18:24:39Z DEBUG] ================================
Welcome to fish, the friendly interactive shell
Type help for instructions on how to use fish
johndoe@hakoniwa ~>
...
```

## Firefox

```console,ignore
$ mkdir -p ~/.local/share/hakoniwa/apps/firefox
$ hakoniwa run -v -c ~/.config/hakoniwa.d/firefox.toml
[2025-03-14T18:25:56Z DEBUG] CONFIG: /home/johndoe/.config/hakoniwa.d/firefox.toml
[2025-03-14T18:25:56Z DEBUG] CONFIG: Including /home/johndoe/.config/hakoniwa.d/abstractions/base.toml
[2025-03-14T18:25:56Z DEBUG] CONFIG: Including /home/johndoe/.config/hakoniwa.d/abstractions/dbus-session.toml
[2025-03-14T18:25:56Z DEBUG] CONFIG: Including /home/johndoe/.config/hakoniwa.d/abstractions/dbus-system.toml
[2025-03-14T18:25:56Z DEBUG] CONFIG: Including /home/johndoe/.config/hakoniwa.d/abstractions/x11.toml
[2025-03-14T18:25:56Z DEBUG] CONFIG: Including /home/johndoe/.config/hakoniwa.d/abstractions/audio.toml
[2025-03-14T18:25:56Z DEBUG] CONFIG: Including /home/johndoe/.config/hakoniwa.d/abstractions/graphics.toml
[2025-03-28T18:25:56Z DEBUG] Unshare namespaces: CloneFlags(CLONE_NEWNS | CLONE_NEWCGROUP | CLONE_NEWUTS | CLONE_NEWIPC | CLONE_NEWUSER | CLONE_NEWPID | CLONE_NEWNET)
[2025-03-28T18:25:56Z DEBUG] RootDir: "/tmp/hakoniwa-obyaKr" -> "/"
[2025-03-28T18:25:56Z DEBUG] Mount: "/usr/bin" -> "/bin", FsType(), MsFlags(MS_RDONLY | MS_NOSUID | MS_BIND | MS_REC)
[2025-03-28T18:25:56Z DEBUG] Mount: "devfs" -> "/dev", FsType(devfs), MsFlags(0x0)
[2025-03-28T18:25:56Z DEBUG] Mount: "/dev/dri" -> "/dev/dri", FsType(), MsFlags(MS_RDONLY | MS_NOSUID | MS_BIND | MS_REC)
[2025-03-28T18:25:56Z DEBUG] Mount: "/dev/snd" -> "/dev/snd", FsType(), MsFlags(MS_RDONLY | MS_NOSUID | MS_BIND | MS_REC)
[2025-03-28T18:25:56Z DEBUG] Mount: "/etc" -> "/etc", FsType(), MsFlags(MS_RDONLY | MS_NOSUID | MS_BIND | MS_REC)
[2025-03-28T18:25:56Z DEBUG] Mount: "/home/johndoe/.local/share/hakoniwa/apps/firefox" -> "/home/johndoe", FsType(), MsFlags(MS_NOSUID | MS_BIND | MS_REC)
[2025-03-28T18:25:56Z DEBUG] Mount: "/home/johndoe/.config/pulse" -> "/home/johndoe/.config/pulse", FsType(), MsFlags(MS_NOSUID | MS_BIND | MS_REC)
[2025-03-28T18:25:56Z DEBUG] Mount: "/home/johndoe/Downloads" -> "/home/johndoe/Downloads", FsType(), MsFlags(MS_NOSUID | MS_BIND | MS_REC)
[2025-03-28T18:25:56Z DEBUG] Mount: "/usr/lib" -> "/lib", FsType(), MsFlags(MS_RDONLY | MS_NOSUID | MS_BIND | MS_REC)
[2025-03-28T18:25:56Z DEBUG] Mount: "/usr/lib" -> "/lib64", FsType(), MsFlags(MS_RDONLY | MS_NOSUID | MS_BIND | MS_REC)
[2025-03-28T18:25:56Z DEBUG] Mount: "proc" -> "/proc", FsType(proc), MsFlags(MS_NOSUID | MS_NODEV | MS_NOEXEC)
[2025-03-28T18:25:56Z DEBUG] Mount: "tmpfs" -> "/run", FsType(tmpfs), MsFlags(MS_NOSUID | MS_NODEV)
[2025-03-28T18:25:56Z DEBUG] Mount: "/run/dbus/system_bus_socket" -> "/run/dbus/system_bus_socket", FsType(), MsFlags(MS_NOSUID | MS_BIND | MS_REC)
[2025-03-28T18:25:56Z DEBUG] Mount: "/run/user/1000/bus" -> "/run/user/1000/bus", FsType(), MsFlags(MS_NOSUID | MS_BIND | MS_REC)
[2025-03-28T18:25:56Z DEBUG] Mount: "/run/user/1000/pipewire-0" -> "/run/user/1000/pipewire-0", FsType(), MsFlags(MS_NOSUID | MS_BIND | MS_REC)
[2025-03-28T18:25:56Z DEBUG] Mount: "/run/user/1000/pulse" -> "/run/user/1000/pulse", FsType(), MsFlags(MS_NOSUID | MS_BIND | MS_REC)
[2025-03-28T18:25:56Z DEBUG] Mount: "/run/user/1000/xauth_XAhFEP" -> "/run/user/1000/xauth_XAhFEP", FsType(), MsFlags(MS_RDONLY | MS_NOSUID | MS_BIND | MS_REC)
[2025-03-28T18:25:56Z DEBUG] Mount: "/usr/bin" -> "/sbin", FsType(), MsFlags(MS_RDONLY | MS_NOSUID | MS_BIND | MS_REC)
[2025-03-28T18:25:56Z DEBUG] Mount: "tmpfs" -> "/sys", FsType(tmpfs), MsFlags(MS_NOSUID | MS_NODEV)
[2025-03-28T18:25:56Z DEBUG] Mount: "/sys/dev/char" -> "/sys/dev/char", FsType(), MsFlags(MS_RDONLY | MS_NOSUID | MS_BIND | MS_REC)
[2025-03-28T18:25:56Z DEBUG] Mount: "/sys/devices" -> "/sys/devices", FsType(), MsFlags(MS_RDONLY | MS_NOSUID | MS_BIND | MS_REC)
[2025-03-28T18:25:56Z DEBUG] Mount: "tmpfs" -> "/tmp", FsType(tmpfs), MsFlags(MS_NOSUID | MS_NODEV)
[2025-03-28T18:25:56Z DEBUG] Mount: "/tmp/.X11-unix" -> "/tmp/.X11-unix", FsType(), MsFlags(MS_RDONLY | MS_NOSUID | MS_BIND | MS_REC)
[2025-03-28T18:25:56Z DEBUG] Mount: "/usr" -> "/usr", FsType(), MsFlags(MS_RDONLY | MS_NOSUID | MS_BIND | MS_REC)
[2025-03-28T18:25:56Z DEBUG] UID mapping: container_id: 1000, host_id: 1000, count: 1
[2025-03-28T18:25:56Z DEBUG] GID mapping: container_id: 1000, host_id: 1000, count: 1
[2025-03-28T18:25:56Z DEBUG] Env: XAUTHORITY=/run/user/1000/xauth_XAhFEP
[2025-03-28T18:25:56Z DEBUG] Env: LANGUAGE=en_US
[2025-03-28T18:25:56Z DEBUG] Env: LANG=en_US.UTF-8
[2025-03-28T18:25:56Z DEBUG] Env: DISPLAY=:1
[2025-03-28T18:25:56Z DEBUG] Env: DBUS_SESSION_BUS_ADDRESS=unix:path=/run/user/1000/bus
[2025-03-28T18:25:56Z DEBUG] Env: HOME=/home/johndoe
[2025-03-28T18:25:56Z DEBUG] Landlock: Load 13 FS rules
[2025-03-28T18:25:56Z DEBUG] Seccomp: Load 439 rules for architectures(X8664, X86, X32)
[2025-03-28T18:25:56Z DEBUG] Execve: "/usr/bin/firefox", []
[2025-03-28T18:25:56Z DEBUG] ================================
[2025-03-28T18:25:56Z DEBUG] Configuring Network: Execve:
    ["pasta", "--config-net", "--no-map-gw", "--tcp-ports", "none", "--udp-ports", "none", "--udp-ns", "none", "-T", "auto", "58927"]
[2025-03-28T18:25:56Z DEBUG] Configuring Network: Output:
    Template interface: wlan0 (IPv4), wlan0 (IPv6)
    Namespace interface: wlan0
    MAC:
        host: 9a:55:9a:55:9a:55
    DHCP:
        assign: 192.168.2.82
        mask: 255.255.255.0
        router: 192.168.2.1
    DNS:
        192.168.2.1
    DNS search list:
        lan
    NDP/DHCPv6:
        assign: 240e:3b7:3273:8de0:e727:43c9:3680:b29e
        router: fe80::1
        our link-local: fe80::1
    DNS:
        fe80::1
    DNS search list:
        lan

[2025-03-28T18:25:56Z DEBUG] ================================
...
```
