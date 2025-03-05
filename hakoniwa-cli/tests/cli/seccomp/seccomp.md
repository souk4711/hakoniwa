# --seccomp

Set seccomp security profile [default: podman]

## podman

```console
$ hakoniwa run -vv
...
[..] UID mapping: container_id: [..]
[..] GID mapping: container_id: [..]
[..] Seccomp: Load 439 rules for architectures([..]X8664[..])
...
[..] Execve: "/bin/sh", []
...
```

## confined

```console
$ hakoniwa run -vv --seccomp=unconfined
...
[..] UID mapping: container_id: [..]
[..] GID mapping: container_id: [..]
[..] Execve: "/bin/sh", []
...
```
