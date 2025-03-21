# HowTo - Launch CLI App

## Fish

```sh
# Create home folder for fish user
export HAKONIWA_DATA_HOME=$HOME/.local/share/hakoniwa
mkdir -p "$HAKONIWA_DATA_HOME/apps/fish"

# Run fish
hakoniwa run -v \
  --unshare-all \
  --devfs /dev --tmpfs /tmp \
  -B "$HAKONIWA_DATA_HOME/apps/fish":"$HOME" -e HOME \
  -e TERM \
  -- fish
```
