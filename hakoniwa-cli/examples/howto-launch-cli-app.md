# HowTo - Launch CLI App

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
