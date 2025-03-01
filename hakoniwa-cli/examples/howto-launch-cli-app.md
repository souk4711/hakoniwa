# HowTo - Launch CLI App

## Bash

```sh
# Create home folder for bash user
export HAKONIWA_DATA_HOME=$HOME/.local/share/hakoniwa
mkdir -p "$HAKONIWA_DATA_HOME/apps/bash"

# Run bash
hakoniwa run \
  --devfs /dev --tmpfs /tmp \
  -B "$HAKONIWA_DATA_HOME/apps/bash":"$HOME" -e HOME \
  -e TERM \
  -- bash
```
