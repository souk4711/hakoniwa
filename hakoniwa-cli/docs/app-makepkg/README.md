# App - makepkg

## Launch With Command Line Arguments

```sh
mkdir -p ~/hakoniwa/apps/makepkg
hakoniwa run -v \
  --unshare-all \
  --rootfs / --devfs /dev --tmpfs /tmp --tmpfs /home -b /var/lib/pacman \
  --network=pasta \
  -e PATH \
  -B "$HOME/hakoniwa/apps/makepkg":"$HOME" -e HOME \
  -w . \
  -- /usr/bin/makepkg
```

- `--unshare-all`
  - Create an isolated environment for the process
- `--rootfs / --devfs /dev --tmpfs /tmp --tmpfs /home -b /var/lib/pacman`
  - Create a new root file system
- `--network=pasta`
  - Access network through `pasta`
- `-e PATH`
  - Set env `PATH` which contains a list of locations that the OS searches for `clang`, `gcc`, etc
- `-B "$HOME/hakoniwa/apps/makepkg":"$HOME" -e HOME`
  - Use `$HOME/hakoniwa/apps/makepkg` as your home folder to make your data (e.g. `~/.cargo`) persistent.
- `-w .`
  - Bind mount current working directory with read-write access
- `-- /usr/bin/makepkg`
  - Run

> [!NOTE]
>
> - If you experience a DNS lookup failure, read [this](../troubleshooting-systemd-resolved) to learn more.
> - If you want access any host-service port, use `--network=pasta:-T,auto`.

## Launch With Config File

```sh
hakoniwa run -v -c /etc/hakoniwa.d/makepkg.toml
```

The config file `makepkg.toml` can be found in [Hakoniwa.d](https://github.com/souk4711/hakoniwa.d).
