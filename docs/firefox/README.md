# Firefox

## Basic

```sh
hakoniwa run -v \
  --unshare-all \
  --rootfs / --devfs /dev --tmpfs /tmp --tmpfs /run --tmpfs /home \
  -b /dev/dri -b /dev/snd -b /sys \
  -b /tmp/.X11-unix -e DISPLAY -b "$XAUTHORITY" -e XAUTHORITY \
  -b /run/dbus/system_bus_socket -b "$XDG_RUNTIME_DIR/bus" -e DBUS_SESSION_BUS_ADDRESS \
  --network=pasta \
  -B "$HOME/Downloads" \
  -- /bin/firefox
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
  - Access network
- `-B "$HOME/Downloads"`
  - Share download folder
- `-- /bin/firefox`
  - Run

## Advanced

### HOME

Use `~/.local/share/hakoniwa/apps/firefox` as your home folder to make your data persistent.

```
export HAKONIWA_DATA_HOME=$HOME/.local/share/hakoniwa
mkdir -p "$HAKONIWA_DATA_HOME/apps/firefox"

hakoniwa run -v \
  --unshare-all \
  --rootfs / --devfs /dev --tmpfs /tmp --tmpfs /run --tmpfs /home \
  -b /dev/dri -b /dev/snd -b /sys \
  -b /tmp/.X11-unix -e DISPLAY -b "$XAUTHORITY" -e XAUTHORITY \
  -b /run/dbus/system_bus_socket -b "$XDG_RUNTIME_DIR/bus" -e DBUS_SESSION_BUS_ADDRESS \
  --network=pasta \
  -B "$HAKONIWA_DATA_HOME/apps/firefox":"$HOME" -e HOME \
  -B "$HOME/Downloads" \
  -- /bin/firefox
```

### Launch Script

```sh
#!/usr/bin/env sh

mkdir -p ~/.local/share/hakoniwa/apps/firefox
exec /usr/bin/hakoniwa run -c ~/.config/hakoniwa.d/firefox.toml -- /bin/firefox "$@"
```

the `firefox.toml` can be found [here](../xdg/config/hakoniwa.d/firefox.toml).
