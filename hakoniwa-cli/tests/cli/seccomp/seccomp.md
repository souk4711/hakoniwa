# --seccomp

Set seccomp security profile [default: podman]

## Use builtin profile `podman`

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

## Use customized profile

```console
$ hakoniwa run -vv --seccomp=./examples/hakoniwa.d/abstractions/seccomp/audit.json
...
[..] UID mapping: container_id: [..]
[..] GID mapping: container_id: [..]
[..] Seccomp: Load 1 rules for architectures([..])
...
[..] Execve: "/bin/sh", []
...
```

## unconfined

```console
$ hakoniwa run -vv --seccomp=unconfined
...
[..] UID mapping: container_id: [..]
[..] GID mapping: container_id: [..]
[..] Execve: "/bin/sh", []
...
```
