# Usage - Misc

## --hostname

Custom hostname in the container (implies **--unshare-uts**)

```console
$ hakoniwa run --hostname myhost -- hostname
myhost

```

## --verbose (alias -v)

Increase logging verbosity (repeatable)

```console,ignore
$ hakoniwa run --verbose -- ls
[2025-04-07T09:23:39Z DEBUG] Unshare namespaces: CloneFlags(CLONE_NEWNS | CLONE_NEWUSER | CLONE_NEWPID)
[2025-04-07T09:23:39Z DEBUG] RootDir: "/tmp/hakoniwa-oKhXnF" -> "/"
[2025-04-07T09:23:39Z DEBUG] Mount: "/etc" -> "/etc", FsType(), MsFlags(MS_RDONLY | MS_NOSUID | MS_BIND | MS_REC)
[2025-04-07T09:23:39Z DEBUG] Mount: "proc" -> "/proc", FsType(proc), MsFlags(MS_NOSUID | MS_NODEV | MS_NOEXEC)
[2025-04-07T09:23:39Z DEBUG] Mount: "/usr" -> "/usr", FsType(), MsFlags(MS_RDONLY | MS_NOSUID | MS_BIND | MS_REC)
[2025-04-07T09:23:39Z DEBUG] FsOperation: symlink: /bin -> usr/bin
[2025-04-07T09:23:39Z DEBUG] FsOperation: symlink: /lib -> usr/lib
[2025-04-07T09:23:39Z DEBUG] FsOperation: symlink: /lib64 -> usr/lib
[2025-04-07T09:23:39Z DEBUG] FsOperation: symlink: /sbin -> usr/bin
[2025-04-07T09:23:39Z DEBUG] UID mapping: container_id: 1000, host_id: 1000, count: 1
[2025-04-07T09:23:39Z DEBUG] GID mapping: container_id: 1000, host_id: 1000, count: 1
[2025-04-07T09:23:39Z DEBUG] Seccomp: Load 439 rules for architectures(X86, X8664, X32)
[2025-04-07T09:23:39Z DEBUG] Execve: "/usr/bin/ls", []
[2025-04-07T09:23:39Z DEBUG] ================================
bin  etc  lib  lib64  proc  sbin  usr
[2025-04-07T09:23:39Z DEBUG] ================================
[2025-04-07T09:23:39Z DEBUG] Exited: Process(/usr/bin/ls) exited with code 0
[2025-04-07T09:23:39Z DEBUG] Rusage: real time: 12.592527ms
[2025-04-07T09:23:39Z DEBUG] Rusage: user time: 9.572ms
[2025-04-07T09:23:39Z DEBUG] Rusage:  sys time: 2.877ms
```
