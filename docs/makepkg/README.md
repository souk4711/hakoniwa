# makepkg

## Basic

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
