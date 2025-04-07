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
  - Access network through `pasta`
- `-e PATH`
  - Set env `PATH` which contains a list of locations that the OS searches for `clang`, `gcc`, etc
- `-w .`
  - Bind mount current working directory with read-write access
- `-- makepkg`
  - Run

## Advanced

### HOME

Use `~/.local/share/hakoniwa/apps/makepkg` as your home folder to make your data (e.g. `~/.cargo`) persistent.

```sh
export HAKONIWA_DATA_HOME=$HOME/.local/share/hakoniwa
mkdir -p "$HAKONIWA_DATA_HOME/apps/makepkg"

hakoniwa run -v \
  --unshare-all \
  --rootfs / --devfs /dev --tmpfs /tmp --tmpfs /home -b /var/lib/pacman \
  --network=pasta \
  -B "$HAKONIWA_DATA_HOME/apps/makepkg":"$HOME" -e HOME \
  -e PATH \
  -w . \
  -- /bin/makepkg
```

> [!NOTE]
> If your want access any host-service port, use `--network=pasta:-T,auto`.

### Launch Script

Create an executable file `~/.local/bin/makepkg` with the following content

```sh
#!/usr/bin/env sh

mkdir -p ~/.local/share/hakoniwa/apps/makepkg
exec /usr/bin/hakoniwa run -c ~/.config/hakoniwa.d/makepkg.toml -- /bin/makepkg "$@"
```

the `makepkg.toml` can be found [here](../xdg/config/hakoniwa.d/makepkg.toml).

### Desktop Integration

Read [XDG](../xdg) to learn more.

## Links

- [makepkg - ArchWiki](https://wiki.archlinux.org/title/Makepkg)
- [Rust package guidelines - ArchWiki](https://wiki.archlinux.org/title/Rust_package_guidelines)
