# --seccomp

Set seccomp security profile [default: podman]

## audit

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

```console
$ hakoniwa run --seccomp=audit -- cat /proc/self/status
...
NoNewPrivs:[..]1
Seccomp:[..]2
Seccomp_filters:[..]1
...
```

## podman

```console
$ hakoniwa run -vv
...
[..] UID mapping: container_id: [..]
[..] GID mapping: container_id: [..]
[..] Seccomp: Load 439 rules for architectures(X32, X86, X8664)
...
[..] Execve: "/bin/sh", []
...
```

```console
$ hakoniwa run -- cat /proc/self/status
...
NoNewPrivs:[..]1
Seccomp:[..]2
Seccomp_filters:[..]1
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

```console
$ hakoniwa run --seccomp=unconfined -- cat /proc/self/status
...
NoNewPrivs:[..]1
Seccomp:[..]0
Seccomp_filters:[..]0
...
```

## customized profile

```console
$ hakoniwa run -vv --seccomp=./tests/fixtures/config/abstractions/seccomp/fine-grained.json
...
[..] UID mapping: container_id: [..]
[..] GID mapping: container_id: [..]
[..] Seccomp: Load 372 rules for architectures(X32, X86, X8664)
...
[..] Execve: "/bin/sh", []
...
```
