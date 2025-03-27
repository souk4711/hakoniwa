# CfgSeccomp

## CfgSeccomp#path `podman`

```console
$ hakoniwa run -vv --config ./tests/fixtures/config/cfg-default.toml
...
[..] Seccomp: Load 439 rules for architectures([..]X8664[..])
...
[..] Execve: "/bin/sh", []
...
```

## CfgSeccomp#path

```console
$ hakoniwa run -vv --config ./tests/fixtures/config/field-seccomp.toml
...
[..] Seccomp: Load 372 rules for architectures([..])
...
[..] Execve: "/bin/sh", []
...
```
