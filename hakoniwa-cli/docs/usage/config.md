# Usage - Config

## --config (alias -c)

Load configuration from a specified file, ignoring all other cli arguments

```console
$ hakoniwa run --config ./docs/howto-create-profile/profiles/example.toml -- dd if=/dev/random of=/tmp/output.txt count=1 bs=4
1+0 records in
1+0 records out
4 bytes copied, [..]

```

Read [HowTo - Create Profile](../howto-create-profile) to learn more.
