# App - Firefox

## Launch With Command Line Arguments

```sh
mkdir -p ~/hakoniwa/apps/firefox
hakoniwa run -v \
  --unshare-all \
  --rootfs / --devfs /dev --tmpfs /tmp --tmpfs /run --tmpfs /home \
  -b /dev/dri -b /dev/snd -b /sys \
  -b /tmp/.X11-unix -e DISPLAY -b "$XAUTHORITY" -e XAUTHORITY \
  -b /run/dbus/system_bus_socket -b "$XDG_RUNTIME_DIR/bus" -e DBUS_SESSION_BUS_ADDRESS \
  --network=pasta \
  -B "$HOME/hakoniwa/apps/firefox":"$HOME" -e HOME \
  -B "$HOME/Downloads" \
  -- /usr/bin/firefox
```

- `--unshare-all`
  - Create an isolated environment for the process
- `--rootfs / --devfs /dev --tmpfs /tmp --tmpfs /run --tmpfs /home`
  - Create a new root file system
- `-b /dev/dri -b /dev/snd -b /sys`
  - Allow Firefox access to GPU and Sound Card
- `-b /tmp/.X11-unix -e DISPLAY -b "$XAUTHORITY" -e XAUTHORITY`
  - Communicates with X Server
- `-b /run/dbus/system_bus_socket -b "$XDG_RUNTIME_DIR/bus" -e DBUS_SESSION_BUS_ADDRESS`
  - Communicates with D-Bus
- `--network=pasta`
  - Access network through `pasta`
- `-B "$HOME/hakoniwa/apps/firefox":"$HOME" -e HOME`
  - Use `$HOME/hakoniwa/apps/firefox` as your home folder to make your data (e.g. `~/.mozilla`) persistent.
- `-B "$HOME/Downloads"`
  - Share `Downloads` folder
- `-- /usr/bin/firefox`
  - Run

> [!NOTE]
>
> - If you experience a DNS lookup failure, read [this](../troubleshooting-systemd-resolved) to learn more.
> - If you want access any host-service port, use `--network=pasta:-T,auto`.

## Launch With Config File

```sh
hakoniwa run -v -c /etc/hakoniwa.d/firefox.toml
```

The config file `firefox.toml` can be found in [Hakoniwa.d](https://github.com/souk4711/hakoniwa.d).
