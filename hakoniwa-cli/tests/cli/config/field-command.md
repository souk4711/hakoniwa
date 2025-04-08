# CfgCommand

## CfgCommand#cmdline

```console
$ hakoniwa run --config ./tests/fixtures/config/field-command.toml
bin
etc
lib
lib64
proc
sbin
usr

```

## CfgCommand#cmdline, CLI ARGS first

```console
$ hakoniwa run --config ./tests/fixtures/config/field-command.toml -- findmnt /
TARGET [..] OPTIONS
...
```

## CfgCommand#cwd

```console
$ hakoniwa run --config ./tests/fixtures/config/field-command.toml -- pwd
/usr/bin

```
