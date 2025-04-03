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
│       ├── wayland.toml        # Wayland
│       ├── x11.toml            # X11
│       └── ...                 # ...
│   ├── firefox.toml            # Profile for App Firefox
│   ├── msedge.toml             # Profile for App Microsoft Edge
│   └── ...                     # Profile for App ...

├── .local/bin/                 #
│   ├── firefox                 # Launch Script for App Firefox
│   ├── microsoft-edge-stable   # Launch Script for App Microsoft Edge
│   └── ...                     # Launch Script for App ...
│
├── .local/share/hakoniwa/      # Hakoniwa App Data
│   └── apps/                   #
│       ├── firefox/            # HOME for App Firefox
│       ├── msedge/             # HOME for App Microsoft Edge
│       └── ...                 # HOME for App ...
```

## Installation

Make sure the `hakoniwa` binary is installed in `/usr/bin`.

```console,ignore
$ file -i /usr/bin/hakoniwa
/usr/bin/hakoniwa: application/x-pie-executable; charset=binary
```

## Profile

Create a hakoniwa profile for your app, e.g. `~/.config/hakoniwa.d/firefox.toml`:

```toml
"@include" = [
  "abstractions/base.toml",
  "abstractions/dbus-session.toml",
  "abstractions/dbus-system.toml",
  "abstractions/wayland.toml",
  "abstractions/audio.toml",
  "abstractions/graphics.toml",
]

mounts = [
  { source = "{{ HOME }}/.local/share/hakoniwa/apps/firefox", destination = "{{ HOME }}", rw = true },
  { source = "{{ HOME }}/Downloads", rw = true },
]

envs = [
  { name = "HOME" },
]

[[landlock.resources]]
type = "tcp.connect"
unrestrict = true

[network]
mode = "pasta"
options = ["-T", "auto"]

[command]
cmdline = ["/bin/firefox"]
cwd = "{{ HOME }}"
```

## Launch Script

Create a launch script for your app, e.g. `~/.local/bin/firefox`:

```sh
#!/usr/bin/env sh

mkdir -p ~/.local/share/hakoniwa/apps/firefox
exec /usr/bin/hakoniwa run -c ~/.config/hakoniwa.d/firefox.toml -- /bin/firefox "$@"
```

Do not forget change permissions:

```sh
chmod +x ~/.local/bin/firefox
```

Run it from terminal:

```console,ignore
$ which firefox
/home/johndoe/.local/bin/firefox

$ firefox
...
```

> Note:
> Path `~/.local/bin` must be set before `/usr/bin` in the PATH environment variable.

## Desktop entries

Check the `/usr/share/applications/*.desktop` files if they contain the full path to the
respective executable, removes the full path. e.g.

```console,ignore
$ grep -r Exec /usr/share/applications/firefox.desktop
Exec=/usr/lib/firefox/firefox %u
Exec=/usr/lib/firefox/firefox --new-window %u
Exec=/usr/lib/firefox/firefox --private-window %u
Exec=/usr/lib/firefox/firefox --ProfileManager

$ sudo vim /user/share/applications/firefox.desktop
...

$ grep -r Exec /usr/share/applications/firefox.desktop
Exec=firefox %u
Exec=firefox --new-window %u
Exec=firefox --private-window %u
Exec=firefox --ProfileManager
```

## Links

- [Firejail - ArchWiki](https://wiki.archlinux.org/title/Firejail)
- [Desktop entries - ArchWiki](https://wiki.archlinux.org/title/Desktop_entries)
