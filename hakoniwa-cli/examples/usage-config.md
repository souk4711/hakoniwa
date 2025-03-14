# Usage - Config

## --config

Load configuration from a specified file, ignoring all other cli arguments

```console
$ hakoniwa run --config ./examples/hakoniwa.d/example.toml -- dd if=/dev/random of=/tmp/output.txt count=1 bs=4
1+0 records in
1+0 records out
4 bytes copied, [..]

```
