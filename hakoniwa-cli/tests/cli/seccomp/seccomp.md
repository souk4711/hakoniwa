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

## Use builtin profile `audit`

```console
$ hakoniwa run -vv --seccomp=audit
...
[..] UID mapping: container_id: [..]
[..] GID mapping: container_id: [..]
[..] Seccomp: Load 1 rules for architectures([..])
...
[..] Execve: "/bin/sh", []
...
```

## Use customized profile

```console
$ hakoniwa run -vv --seccomp=./tests/fixtures/config/abstractions/seccomp/fine-grained.json
...
[..] UID mapping: container_id: [..]
[..] GID mapping: container_id: [..]
[..] Seccomp: Load 372 rules for architectures([..])
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
