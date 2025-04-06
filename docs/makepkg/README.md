# makepkg

## Basic

```sh
hakoniwa run --unshare-all --network=pasta --rootfs / --devfs /dev --tmpfs /tmp --tmpfs /home -b /var/lib/pacman -e PATH -w . -- /bin/makepkg
```

- `--unshare-all`
  - Create an isolated environment for the process
- `--rootfs / --devfs /dev --tmpfs /tmp --tmpfs /home -b /var/lib/pacman`
  - Create a new root file system
- `--network=pasta`
  - Use Pasta network
- `-e PATH`
  - Set env `PATH` which contains a list of locations that the OS searches for `clang`, `gcc`, etc
- `-w .`
  - Bind mount current working directory with read-write access
- `-- makepkg`
  - Run

## Advanced

### HTTP_PROXY

```sh
hakoniwa run --unshare-all --network=pasta --rootfs / --devfs /dev --tmpfs /tmp --tmpfs /home -b /var/lib/pacman -e PATH -w . \
  -e ALL_PROXY -e HTTP_PROXY -e HTTPS_PROXY -- /bin/makepkg
```

> [!NOTE]
> If the proxy server is running on your local host, donot forget to use `--network=pasta:-T,auto`.

### Shell Wrapper

```sh
#!/usr/bin/env sh

mkdir -p ~/.local/share/hakoniwa/apps/makepkg
exec /usr/bin/hakoniwa run -c ~/.config/hakoniwa.d/makepkg.toml -- /bin/makepkg "$@"
```

the `makepkg.toml` can be found [here](../xdg/config/hakoniwa.d/makepkg.toml).

## Links

- [makepkg - ArchWiki](https://wiki.archlinux.org/title/Makepkg)
- [Rust package guidelines - ArchWiki](https://wiki.archlinux.org/title/Rust_package_guidelines)
