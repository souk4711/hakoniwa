# HowTo - Launch Desktop App

## Firefox

```sh
# Create home folder for firefox user
export HAKONIWA_DATA_HOME=$HOME/.local/share/hakoniwa
mkdir -p $HAKONIWA_DATA_HOME/apps/firefox

# Run firefox
hakoniwa run \
  --devfs /dev -b /dev/dri -b /dev/snd -b /sys \
  --tmpfs /tmp -b /tmp/.X11-unix -e DISPLAY \
  --tmpfs /run -b $XDG_RUNTIME_DIR/bus -e DBUS_SESSION_BUS_ADDRESS \
  -B $HAKONIWA_DATA_HOME/apps/firefox:$HOME -e HOME \
  -B $HOME/Downloads \
  -- firefox
```

More explain:

- `--devfs /dev -b /dev/dri -b /dev/snd -b /sys`
  - Allow Firefox access to GPU and Sound Card
- `--tmpfs /tmp -b /tmp/.X11-unix -e DISPLAY`
  - Communicates with X Server
- `--tmpfs /run -b $XDG_RUNTIME_DIR/bus -e DBUS_SESSION_BUS_ADDRESS`
  - Communicates with D-Bus
- `-B $HAKONIWA_DATA_HOME/apps/firefox:$HOME -e HOME`
  - Use ~/.local/share/hakoniwa/apps/firefox as home folder
- `-B $HOME/Downloads`
  - Share download folder
- `-- firefox`
  - Run
