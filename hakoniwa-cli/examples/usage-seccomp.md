# Usage - Seccomp Profile

## --seccomp

Set seccomp security profile [default: podman]

```console,ignore
$ # Use builtin profile
$ hakoniwa run -vv
...
[2025-03-05T14:00:35Z DEBUG] Seccomp: Load 439 rules for architectures(X86, X8664, X32)
[2025-03-05T14:00:35Z TRACE] Seccomp rule: ... -> Errno(38)
[2025-03-05T14:00:35Z TRACE] Seccomp rule: bdflush(...) -> Errno(1)
...

$ # Use customized profile
$ hakoniwa run -vv --seccomp=./examples/hakoniwa.d/abstractions/seccomp/audit.json
...
[2025-03-05T13:59:46Z DEBUG] Seccomp: Load 1 rules for architectures(X86, X32, X8664)
[2025-03-05T13:59:46Z TRACE] Seccomp rule: ... -> Log
[2025-03-05T13:59:46Z DEBUG] Execve: "/bin/sh", []
...

$ # Disable seccomp
$ hakoniwa run --seccomp=unconfined
...
[2025-03-05T14:01:39Z DEBUG] Execve: "/bin/sh", []
...
```

Read [Introduction to Seccomp Profile](./howto-introduction-to-seccomp-profile.md) to learn more.
