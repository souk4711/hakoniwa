# CfgLimit

## fsize `file too large`

```console
$ hakoniwa run --config ./tests/fixtures/config/field-limits.toml -- dd if=/dev/random of=/tmp/output.txt count=1 bs=4
? 1
[..]: File too large
...

```

## walltime `killed in 2s`

```console
$ hakoniwa run --config ./tests/fixtures/config/field-limits.toml -- sleep 5
? 137
hakoniwa: process(/usr/bin/sleep) received signal SIGKILL

```
