# HowTo - Launch CLI/Desktop App

## Fish

```sh
# Create home folder for fish
export HAKONIWA_DATA_HOME=$HOME/.local/share/hakoniwa
mkdir -p "$HAKONIWA_DATA_HOME/apps/fish"

# Run fish
hakoniwa run -v \
  --unshare-all \
  --devfs /dev --tmpfs /tmp \
  -B "$HAKONIWA_DATA_HOME/apps/fish":"$HOME" -e HOME \
  -e TERM \
  -- /bin/fish
```

## Darkhttpd

```sh
# Create home folder for darkhttpd
export HAKONIWA_DATA_HOME=$HOME/.local/share/hakoniwa
mkdir -p "$HAKONIWA_DATA_HOME/apps/darkhttpd"

# Run darkhttpd
hakoniwa run -v \
  --unshare-all --network=pasta:-t,8080\
  --devfs /dev \
  -B "$HAKONIWA_DATA_HOME/apps/darkhttpd":"$HOME" -e HOME \
  -b $PWD:/var/www/html -w :/var/www/html \
  -- /bin/darkhttpd .
```

## Firefox

```sh
# Create home folder for firefox
export HAKONIWA_DATA_HOME=$HOME/.local/share/hakoniwa
mkdir -p "$HAKONIWA_DATA_HOME/apps/firefox"

# Run firefox
hakoniwa run -v \
  --unshare-all --network=pasta \
  --devfs /dev -b /dev/dri -b /dev/snd -b /sys \
  --tmpfs /tmp -b /tmp/.X11-unix -e DISPLAY -b "$XAUTHORITY" -e XAUTHORITY \
  --tmpfs /run -b /run/dbus/system_bus_socket -b "$XDG_RUNTIME_DIR/bus" -e DBUS_SESSION_BUS_ADDRESS \
  -B "$HAKONIWA_DATA_HOME/apps/firefox":"$HOME" -e HOME \
  -B "$HOME/Downloads" \
  -- /bin/firefox
```
