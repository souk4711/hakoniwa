# APP - makepkg

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
- `-- /bin/makepkg`
  - Run

> [!NOTE]
>
> - If you receive `Command "pasta" not found`, make sure you have [passt](https://passt.top/passt/about/) installed.
> - If you experience a DNS lookup failure, read [this](../troubleshooting-systemd-resolved) to learn more.
> - If you want access any host-service port, use `--network=pasta:-T,auto`.

## Advanced

### Home Folder

Use `~/.local/share/hakoniwa/apps/makepkg` as your home folder to make your data (e.g. `~/.cargo`) persistent.

```sh
export HAKONIWA_DATA_HOME=$HOME/.local/share/hakoniwa
mkdir -p "$HAKONIWA_DATA_HOME/apps/makepkg"

hakoniwa run -v \
  --unshare-all \
  --rootfs / --devfs /dev --tmpfs /tmp --tmpfs /home -b /var/lib/pacman \
  --network=pasta \
  -e PATH \
  -w . \
  -B "$HAKONIWA_DATA_HOME/apps/makepkg":"$HOME" -e HOME \
  -- /bin/makepkg
```

### Desktop Integration

Read [Hakoniwa.d](https://github.com/souk4711/hakoniwa.d) to learn more.
