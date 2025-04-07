# XDG - Desktop Integration

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
│
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

> [!NOTE]
> If you want all users to be able to run the app just by with its name, you can put the
> profiles in `/etc/hakoniwa.d`, and put the launch scripts in `/usr/local/bin`.

## Prerequisites

Make sure the `hakoniwa` binary is installed in `/usr/bin`.

```console,ignore
$ file -i /usr/bin/hakoniwa
/usr/bin/hakoniwa: application/x-pie-executable; charset=binary
```

Also check that the path `~/.local/bin` is set before `/usr/bin` in the **PATH** environment variable.

```console,ignore
$ printenv PATH
/home/johndoe/.local/bin:/usr/local/sbin:/usr/local/bin:/usr/bin
```

If not, add following line to `~/.bash_profile`:

```sh
[[ -d "$HOME/.local/bin" ]] && export PATH="$HOME/.local/bin:$PATH"
```

## Profile

Create a hakoniwa profile for your app. E.g. `~/.config/hakoniwa.d/firefox.toml`:

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

Create a launch script for your app. E.g. `~/.local/bin/firefox`:

```sh
#!/usr/bin/env sh

mkdir -p ~/.local/share/hakoniwa/apps/firefox
exec /usr/bin/hakoniwa run -c ~/.config/hakoniwa.d/firefox.toml -- /bin/firefox "$@"
```

Do not forget change permissions:

```sh
chmod +x ~/.local/bin/firefox
```

Now, you can launch it from terminal:

```console,ignore
$ which firefox
/home/johndoe/.local/bin/firefox

$ firefox
...
```

## Desktop entries

Check the `/usr/share/applications/*.desktop` files if they contain the full path to the
respective executable, removes the full path. E.g.

```console,ignore
$ grep -r Exec /usr/share/applications/firefox.desktop
Exec=/usr/lib/firefox/firefox %u
Exec=/usr/lib/firefox/firefox --new-window %u
Exec=/usr/lib/firefox/firefox --private-window %u
Exec=/usr/lib/firefox/firefox --ProfileManager

$ sudo sed -i -e 's/Exec=.*firefox /Exec=firefox /g' /usr/share/applications/firefox.desktop

$ grep -r Exec /usr/share/applications/firefox.desktop
Exec=firefox %u
Exec=firefox --new-window %u
Exec=firefox --private-window %u
Exec=firefox --ProfileManager
```

You should check these files after the relative packages have been updated, use hooks to do this
automatically. For Arch Linux, create a file `/usr/share/libalpm/hooks/hakoniwa.hook` with the
following contents:

```
[Trigger]
Type = Path
Operation = Install
Operation = Upgrade
Target = usr/share/applications/*.desktop

[Action]
Description = Updating the desktop file in /usr/share/applications...
When = PostTransaction
Exec = /usr/share/libalpm/scripts/hakoniwa
```

the `scripts/hakoniwa` can be found [here](./pm/libalpm/scripts/hakoniwa).

Now, you can launch the sandboxed firefox from the start menu.

## Links

- [Firejail - ArchWiki](https://wiki.archlinux.org/title/Firejail)
- [Desktop entries - ArchWiki](https://wiki.archlinux.org/title/Desktop_entries)
