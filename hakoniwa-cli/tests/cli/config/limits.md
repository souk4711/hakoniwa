# CfgLimit

## fsize `2`

```console
$ hakoniwa run --config ./tests/config/limits.toml -- dd if=/dev/random of=/tmp/output.txt count=1 bs=4
? 1
[..]: error writing '/tmp/output.txt': File too large
1+0 records in
0+0 records out
2 bytes copied, [..]

```

## walltime `2`

```console
$ hakoniwa run --config ./tests/config/limits.toml -- sleep 5
? 137
hakoniwa: Process(/usr/bin/sleep) received signal SIGKILL

```
