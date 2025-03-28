# Usage - Misc

## --uidmap

Custom UID in the container

```console,ignore
$ hakoniwa run --uidmap 0 -- id
uid=0(root) gid=1000(johndoe) groups=1000(johndoe),65534(nobody)

```

## --gidmap

Custom GID in the container

```console,ignore
$ hakoniwa run --gidmap 0 -- id
uid=1000(johndoe) gid=0(root) groups=0(root),65534(nobody)

```

## --hostname

Custom hostname in the container (implies --unshare-uts)

```console
$ hakoniwa run --hostname myhost -- hostname
myhost

```

## --verbose

Increase logging verbosity (repeatable)

```console,ignore
$ hakoniwa run --verbose -- ls
[2025-03-28T09:58:05Z DEBUG] Unshare namespaces: CloneFlags(CLONE_NEWNS | CLONE_NEWUSER | CLONE_NEWPID)
[2025-03-28T09:58:05Z DEBUG] RootDir: "/tmp/hakoniwa-KnOF2r" -> "/"
[2025-03-28T09:58:05Z DEBUG] Mount: "/bin" -> "/bin", FsType(), MsFlags(MS_RDONLY | MS_NOSUID | MS_BIND | MS_REC)
[2025-03-28T09:58:05Z DEBUG] Mount: "/etc" -> "/etc", FsType(), MsFlags(MS_RDONLY | MS_NOSUID | MS_BIND | MS_REC)
[2025-03-28T09:58:05Z DEBUG] Mount: "/lib" -> "/lib", FsType(), MsFlags(MS_RDONLY | MS_NOSUID | MS_BIND | MS_REC)
[2025-03-28T09:58:05Z DEBUG] Mount: "/lib64" -> "/lib64", FsType(), MsFlags(MS_RDONLY | MS_NOSUID | MS_BIND | MS_REC)
[2025-03-28T09:58:05Z DEBUG] Mount: "proc" -> "/proc", FsType(proc), MsFlags(MS_NOSUID | MS_NODEV | MS_NOEXEC)
[2025-03-28T09:58:05Z DEBUG] Mount: "/sbin" -> "/sbin", FsType(), MsFlags(MS_RDONLY | MS_NOSUID | MS_BIND | MS_REC)
[2025-03-28T09:58:05Z DEBUG] Mount: "/usr" -> "/usr", FsType(), MsFlags(MS_RDONLY | MS_NOSUID | MS_BIND | MS_REC)
[2025-03-28T09:58:05Z DEBUG] UID mapping: container_id: 1000, host_id: 1000, count: 1
[2025-03-28T09:58:05Z DEBUG] GID mapping: container_id: 1000, host_id: 1000, count: 1
[2025-03-28T09:58:05Z DEBUG] Seccomp: Load 439 rules for architectures(X8664, X86, X32)
[2025-03-28T09:58:05Z DEBUG] Execve: "/usr/bin/ls", []
[2025-03-28T09:58:05Z DEBUG] ================================
bin  etc  lib  lib64  proc  sbin  usr
[2025-03-28T09:58:05Z DEBUG] ================================
[2025-03-28T09:58:05Z DEBUG] Exited: Process(/usr/bin/ls) exited with code 0
[2025-03-28T09:58:05Z DEBUG] Rusage: real time: 17.875582ms
[2025-03-28T09:58:05Z DEBUG] Rusage: user time: 14.103ms
[2025-03-28T09:58:05Z DEBUG] Rusage:  sys time: 1.992ms
```
