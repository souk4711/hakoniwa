# Usage - Seccomp Profile

## --seccomp

Set seccomp security profile [default: **podman**]

> [!WARNING]
> The default seccomp profile has a large ruleset that may affect the performance
> of syscall-heavy apps.

### audit

```console
$ hakoniwa run -vv --seccomp=audit
...
[..] Seccomp: Load 1 rules for architectures([..])
[..] Seccomp rule: ... -> Log
...
```

### podman

```console
$ hakoniwa run -vv --seccomp=podman
...
[..] Seccomp: Load 439 rules for architectures([..])
[..] Seccomp rule: ... -> Errno(38)
[..] Seccomp rule: bdflush(...) -> Errno(1)
...
```

### unconfined

```console
$ hakoniwa run -vv --seccomp=unconfined
...
```

### customized profile

```console
$ hakoniwa run -vv --seccomp=./docs/howto-create-profile/profiles/seccomp/fine-grained.json
...
[..] Seccomp: Load 372 rules for architectures([..])
[..] Seccomp rule: ... -> Errno(0)
[..] Seccomp rule: _llseek(...) -> Allow
...
```
