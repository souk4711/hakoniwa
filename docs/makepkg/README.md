# makepkg

## Basic

hakoniwa run -v \
 --unshare-all \
 --rootfs / --devfs /dev --tmpfs /tmp --tmpfs /run \
 -b /dev/dri -b /dev/snd -b /sys \
 -b /tmp/.X11-unix -e DISPLAY -b "$XAUTHORITY" -e XAUTHORITY \
  -b /run/dbus/system_bus_socket -b "$XDG_RUNTIME_DIR/bus" -e DBUS_SESSION_BUS_ADDRESS \
 --network=pasta \
 -B "$HOME/Downloads" \
 -- /bin/firefox

```sh
hakoniwa run -v \
  --unshare-all \
  --rootfs / --devfs /dev --tmpfs /tmp --tmpfs /home -b /var/lib/pacman \
  --network=pasta \
  -e PATH \
  -w . \
  -- /bin/makepkg
```

- `--unshare-all`
  - Create an isolated environment for the process
- `--rootfs / --devfs /dev --tmpfs /tmp --tmpfs /home -b /var/lib/pacman`
  - Create a new root file system
- `--network=pasta`
  - Access network
- `-e PATH`
  - Set env `PATH` which contains a list of locations that the OS searches for `clang`, `gcc`, etc
- `-w .`
  - Bind mount current working directory with read-write access
- `-- makepkg`
  - Run

## Advanced

### Proxy

```sh
hakoniwa run -v \
  --unshare-all \
  --rootfs / --devfs /dev --tmpfs /tmp --tmpfs /home -b /var/lib/pacman \
  --network=pasta \
  -e PATH -e ALL_PROXY -e HTTP_PROXY -e HTTPS_PROXY \
  -w . \
  -- /bin/makepkg

```

> [!NOTE]
> If the proxy server is running on your local host, donot forget to use `--network=pasta:-T,auto`.

### Launch Script

```sh
#!/usr/bin/env sh

mkdir -p ~/.local/share/hakoniwa/apps/makepkg
exec /usr/bin/hakoniwa run -c ~/.config/hakoniwa.d/makepkg.toml -- /bin/makepkg "$@"
```

the `makepkg.toml` can be found [here](../xdg/config/hakoniwa.d/makepkg.toml).

## Links

- [makepkg - ArchWiki](https://wiki.archlinux.org/title/Makepkg)
- [Rust package guidelines - ArchWiki](https://wiki.archlinux.org/title/Rust_package_guidelines)
