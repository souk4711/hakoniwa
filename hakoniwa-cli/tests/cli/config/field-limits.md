# CfgLimit

## fsize `file too large`

```console
$ hakoniwa run --config ./tests/config/field-limits.toml -- dd if=/dev/random of=/tmp/output.txt count=1 bs=4
? 1
[..]: error writing '/tmp/output.txt': File too large
1+0 records in
0+0 records out
2 bytes copied, [..]

```

## walltime `killed in 2s`

```console
$ hakoniwa run --config ./tests/config/field-limits.toml -- sleep 5
? 137
hakoniwa: Process(/usr/bin/sleep) received signal SIGKILL

```
