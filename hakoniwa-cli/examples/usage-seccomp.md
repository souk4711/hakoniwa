# Usage - Seccomp Profile

## --seccomp

Set seccomp security profile [default: **podman**]

### podman

```console
$ hakoniwa run -vv
...
[..] Seccomp: Load 439 rules for architectures([..])
[..] Seccomp rule: ... -> Errno(38)
[..] Seccomp rule: bdflush(...) -> Errno(1)
...
```

### audit

```console
$ hakoniwa run -vv --seccomp=audit
...
[..] Seccomp: Load 1 rules for architectures([..])
[..] Seccomp rule: ... -> Log
...
```

### unconfined

```console
$ hakoniwa run -vv --seccomp=unconfined
...
```

### customized profile

```console
$ hakoniwa run -vv --seccomp=./examples/hakoniwa.d/abstractions/seccomp/fine-grained.json
...
[..] Seccomp: Load 372 rules for architectures([..])
[..] Seccomp rule: ... -> Errno(0)
[..] Seccomp rule: _llseek(...) -> Allow
...
```
