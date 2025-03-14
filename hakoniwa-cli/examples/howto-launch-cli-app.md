# HowTo - Launch CLI App

## Fish

```sh
# Create home folder for bash user
export HAKONIWA_DATA_HOME=$HOME/.local/share/hakoniwa
mkdir -p "$HAKONIWA_DATA_HOME/apps/fish"

# Run bash
hakoniwa run -vv \
  --devfs /dev --tmpfs /tmp \
  -B "$HAKONIWA_DATA_HOME/apps/fish":"$HOME" -e HOME \
  -e TERM \
  -- bash
```
