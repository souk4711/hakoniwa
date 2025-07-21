# Usage - Misc

## --hostname

Custom hostname in the container (implies **--unshare-uts**)

```console
$ hakoniwa run --hostname myhost -- hostname
myhost

```

## --allow-new-privs

Set the **NoNewPrivileges** flag to off

```console,ignore
$ hakoniwa run --allow-new-privs -- cat /proc/self/status
...
NoNewPrivs:     0
Seccomp:        2
Seccomp_filters:        1
...
```

## --verbose (alias -v)

Increase logging verbosity (repeatable)

```console,ignore
$ hakoniwa run --verbose -- ls
[2025-07-14T13:00:04Z DEBUG] Unshare namespaces: CloneFlags(CLONE_NEWNS | CLONE_NEWUSER | CLONE_NEWPID)
[2025-07-14T13:00:04Z DEBUG] Mount:    root: /tmp/hakoniwa-vMv9k9
[2025-07-14T13:00:04Z DEBUG] Mount: bind-ro: /etc -> /etc
[2025-07-14T13:00:04Z DEBUG] Mount:    proc: /proc
[2025-07-14T13:00:04Z DEBUG] Mount: bind-ro: /usr -> /usr
[2025-07-14T13:00:04Z DEBUG] FsOperation: symlink: /bin -> usr/bin
[2025-07-14T13:00:04Z DEBUG] FsOperation: symlink: /lib -> usr/lib
[2025-07-14T13:00:04Z DEBUG] FsOperation: symlink: /lib32 -> usr/lib32
[2025-07-14T13:00:04Z DEBUG] FsOperation: symlink: /lib64 -> usr/lib
[2025-07-14T13:00:04Z DEBUG] FsOperation: symlink: /sbin -> usr/bin
[2025-07-14T13:00:04Z DEBUG] UID mapping: container_id: 1000, host_id: 1000, count: 1
[2025-07-14T13:00:04Z DEBUG] GID mapping: container_id: 1000, host_id: 1000, count: 1
[2025-07-14T13:00:04Z DEBUG] Seccomp: Load 439 rules for architectures(X8664, X86, X32)
[2025-07-14T13:00:04Z DEBUG] Execve: "/usr/bin/ls", []
[2025-07-14T13:00:04Z DEBUG] ================================
bin  etc  lib  lib32  lib64  proc  sbin  usr
[2025-07-14T13:00:04Z DEBUG] ================================
[2025-07-14T13:00:04Z DEBUG] Exited: process(/usr/bin/ls) exited with code 0
[2025-07-14T13:00:04Z DEBUG] Metric:      RealTime:  0.014707706 sec
[2025-07-14T13:00:04Z DEBUG] Metric:      UserTime:     0.013483 sec
[2025-07-14T13:00:04Z DEBUG] Metric:       SysTime:     0.001038 sec
```
