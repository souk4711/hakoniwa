# XDG

## File Structure

```console,ignore
.                               # HOME
├── .config/hakoniwa.d/         # Hakoniwa Configuration Files
│   └── abstractions/           #
│       ├── audio.toml          # Sound Card, Pulse Audio, etc.
│       ├── base.toml           # Base Sandbox Environment - unshare-all, rootfs, etc.
│       ├── dbus-session.toml   # DBus - session bus
│       ├── dbus-system.toml    # DBus - system bus
│       ├── graphics.toml       # DRI, etc.
│       ├── x11.toml            # X11
│       └── ...                 # ...
│   ├── firefox.toml            # Profile for App Firefox
│   ├── fish.toml               # Profile for App Fish
│   └── ...                     # Profile for App ...
│
├── .local/bin/                 #
│   ├── firefox                 # Launch Script for App Firefox
│   ├── fish                    # Launch Script for App Fish
│   └── ...                     # Launch Script for App ...
│
├── .local/share/hakoniwa/      # Hakoniwa App Data
│   └── apps/                   #
│       ├── firefox/            # HOME for App Firefox
│       ├── fish/               # HOME for App Fish
│       └── ...                 # HOME for App ...
```

## Installation

Make sure the `hakoniwa` binary is installed in `/usr/bin`.

```console,ignore
$ file -i /usr/bin/hakoniwa
/usr/bin/hakoniwa: application/x-pie-executable; charset=binary
```

## Profile

## Launch Script

```sh
[[ -d "$HOME/.local/bin" ]] && export PATH="$HOME/.local/bin:$PATH"
```

## Desktop entries

## Links

- [Firejail - ArchWiki](https://wiki.archlinux.org/title/Firejail)
- [Desktop entries - ArchWiki](https://wiki.archlinux.org/title/Desktop_entries)
