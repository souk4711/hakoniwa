# CfgSeccomp

## CfgSeccomp#path `podman`

```console
$ hakoniwa run -vv --config ./tests/config/cfg-default.toml
...
[..] UID mapping: container_id: [..]
[..] GID mapping: container_id: [..]
[..] Seccomp: Load 439 rules for architectures([..]X8664[..])
...
[..] Execve: "/bin/sh", []
...
```

## CfgSeccomp#path

```console
$ hakoniwa run -vv --config ./tests/config/field-seccomp.toml
...
[..] UID mapping: container_id: [..]
[..] GID mapping: container_id: [..]
[..] Seccomp: Load 1 rules for architectures([..])
[..] Seccomp rule: [..] -> Log
[..] Execve: "/bin/sh", []
...
```
