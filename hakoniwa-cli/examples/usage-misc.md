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

Increase logging verbosity

```console,ignore
$ hakoniwa run --verbose -- true
[2025-02-26T16:12:58Z DEBUG] Unshare namespaces: CloneFlags(CLONE_NEWNS | CLONE_NEWUSER | CLONE_NEWPID)
[2025-02-26T16:12:58Z DEBUG] RootDir: "/tmp/hakoniwa-KqdDmr" -> "/"
[2025-02-26T16:12:58Z DEBUG] Mount: "/bin" -> "/bin", FsType(), MsFlags(MS_RDONLY | MS_NOSUID | MS_BIND | MS_REC)
[2025-02-26T16:12:58Z DEBUG] Mount: "/etc" -> "/etc", FsType(), MsFlags(MS_RDONLY | MS_NOSUID | MS_BIND | MS_REC)
[2025-02-26T16:12:58Z DEBUG] Mount: "/lib" -> "/lib", FsType(), MsFlags(MS_RDONLY | MS_NOSUID | MS_BIND | MS_REC)
[2025-02-26T16:12:58Z DEBUG] Mount: "/lib64" -> "/lib64", FsType(), MsFlags(MS_RDONLY | MS_NOSUID | MS_BIND | MS_REC)
[2025-02-26T16:12:58Z DEBUG] Mount: "proc" -> "/proc", FsType(proc), MsFlags(MS_NOSUID | MS_NODEV | MS_NOEXEC)
[2025-02-26T16:12:58Z DEBUG] Mount: "/sbin" -> "/sbin", FsType(), MsFlags(MS_RDONLY | MS_NOSUID | MS_BIND | MS_REC)
[2025-02-26T16:12:58Z DEBUG] Mount: "/usr" -> "/usr", FsType(), MsFlags(MS_RDONLY | MS_NOSUID | MS_BIND | MS_REC)
[2025-02-26T16:12:58Z DEBUG] UID mapping: container_id: 1000, host_id: 1000, count: 1
[2025-02-26T16:12:58Z DEBUG] GID mapping: container_id: 1000, host_id: 1000, count: 1
[2025-02-26T16:12:58Z DEBUG] Execve: "/usr/bin/true", []
[2025-02-26T16:12:58Z DEBUG] Exited: Process(/usr/bin/true) exited with code 0
[2025-02-26T16:12:58Z DEBUG] Rusage: real time: 370.657µs
[2025-02-26T16:12:58Z DEBUG] Rusage: user time: 0ns
[2025-02-26T16:12:58Z DEBUG] Rusage:  sys time: 345µs
```
