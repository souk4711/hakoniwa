# HowTo - Create Profile

## Example

Create a configuration file:

```toml
# ./examples/hakoniwa.d/example.toml

# unshare linux namespace
namespaces = [
  { type = "cgroup"     },  # --unshare-cgroup
  { type = "ipc"        },  # --unshare-ipc
  { type = "network"    },  # --unshare-network
  { type = "uts"        },  # --unshare-uts
]

# mount filesystem
mounts = [
  { source = "/bin"     },  # --bindmount-ro /bin
  { source = "/etc"     },  # --bindmount-ro /etc
  { source = "/lib"     },  # --bindmount-ro /lib
  { source = "/lib64"   },  # --bindmount-ro /lib64
  { source = "/sbin"    },  # --bindmount-ro /sbin
  { source = "/usr"     },  # --bindmount-ro /usr
  { source = ""          , destination = "/dev" , type = "devfs" },   # --devfs /dev
  { source = ""          , destination = "/tmp" , type = "tmpfs" },   # --tmpfs /tmp
  { source = ""          , destination = "/run" , type = "tmpfs" },   # --tmpfs /run
  { source = "{{ PWD }}" , destination = "/data",   rw = true    },   # --bindmount-rw $PWD
]

# environment
envs = [
  { name = "LANG"                           },  # --setenv LANG
  { name = "LANGUAGE"                       },  # --setenv LANGUAGE
  { name = "TERM"                           },  # --setenv TERM
  { name = "VAR123"      , value = "456"    },  # --setenv VAR123=456
]

# resource limit
limits = [
  { type = "as"          , value = 64000000 },  # --limit-as 64000000
  { type = "walltime"    , value = 60       },  # --limit-walltime 60
]

# seccomp
[seccomp]
path = "{{ __dir__ }}/abstractions/seccomp/fine-grained.json"         # --seccomp ./examples/hakoniwa.d/abstractions/seccomp/fine-grained.json

# cmdline
[command]
cmdline = ["/bin/bash"]
cwd = "/data"
```

Run:

```console,ignore
$ hakoniwa run -v -c ./examples/hakoniwa.d/example.toml
[2025-04-03T15:32:55Z DEBUG] CONFIG: ./examples/hakoniwa.d/example.toml
[2025-04-03T15:32:55Z DEBUG] Unshare namespaces: CloneFlags(CLONE_NEWNS | CLONE_NEWCGROUP | CLONE_NEWUTS | CLONE_NEWIPC | CLONE_NEWUSER | CLONE_NEWPID | CLONE_NEWNET)
[2025-04-03T15:32:55Z DEBUG] RootDir: "/tmp/hakoniwa-6Vy9RB" -> "/"
[2025-04-03T15:32:55Z DEBUG] Mount: "/usr/bin" -> "/bin", FsType(), MsFlags(MS_RDONLY | MS_NOSUID | MS_BIND | MS_REC)
[2025-04-03T15:32:55Z DEBUG] Mount: "/home/johndoe/..." -> "/data", FsType(), MsFlags(MS_NOSUID | MS_BIND | MS_REC)
[2025-04-03T15:32:55Z DEBUG] Mount: "devfs" -> "/dev", FsType(devfs), MsFlags(0x0)
[2025-04-03T15:32:55Z DEBUG] Mount: "/etc" -> "/etc", FsType(), MsFlags(MS_RDONLY | MS_NOSUID | MS_BIND | MS_REC)
[2025-04-03T15:32:55Z DEBUG] Mount: "/usr/lib" -> "/lib", FsType(), MsFlags(MS_RDONLY | MS_NOSUID | MS_BIND | MS_REC)
[2025-04-03T15:32:55Z DEBUG] Mount: "/usr/lib" -> "/lib64", FsType(), MsFlags(MS_RDONLY | MS_NOSUID | MS_BIND | MS_REC)
[2025-04-03T15:32:55Z DEBUG] Mount: "proc" -> "/proc", FsType(proc), MsFlags(MS_NOSUID | MS_NODEV | MS_NOEXEC)
[2025-04-03T15:32:55Z DEBUG] Mount: "tmpfs" -> "/run", FsType(tmpfs), MsFlags(MS_NOSUID | MS_NODEV)
[2025-04-03T15:32:55Z DEBUG] Mount: "/usr/bin" -> "/sbin", FsType(), MsFlags(MS_RDONLY | MS_NOSUID | MS_BIND | MS_REC)
[2025-04-03T15:32:55Z DEBUG] Mount: "tmpfs" -> "/tmp", FsType(tmpfs), MsFlags(MS_NOSUID | MS_NODEV)
[2025-04-03T15:32:55Z DEBUG] Mount: "/usr" -> "/usr", FsType(), MsFlags(MS_RDONLY | MS_NOSUID | MS_BIND | MS_REC)
[2025-04-03T15:32:55Z DEBUG] UID mapping: container_id: 1000, host_id: 1000, count: 1
[2025-04-03T15:32:55Z DEBUG] GID mapping: container_id: 1000, host_id: 1000, count: 1
[2025-04-03T15:32:55Z DEBUG] Env: LANG=en_US.UTF-8
[2025-04-03T15:32:55Z DEBUG] Env: LANGUAGE=en_US
[2025-04-03T15:32:55Z DEBUG] Env: TERM=xterm-256color
[2025-04-03T15:32:55Z DEBUG] Env: VAR123=456
[2025-04-03T15:32:55Z DEBUG] Seccomp: Load 372 rules for architectures(X8664, X86, X32)
[2025-04-03T15:32:55Z DEBUG] Execve: "/bin/bash", []
[2025-04-03T15:32:55Z DEBUG] ================================
[johndoe@hakoniwa data]$ ls
Cargo.toml  examples  LICENSE  src  tests
[johndoe@hakoniwa data]$ env
LANGUAGE=en_US
PWD=/data
LANG=en_US.UTF-8
VAR123=456
TERM=xterm-256color
SHLVL=1
_=/usr/bin/env
[johndoe@hakoniwa data]$ ps aux
USER         PID %CPU %MEM    VSZ   RSS TTY      STAT START   TIME COMMAND
johndoe        1  0.1  0.0  12192  9464 ?        S    23:32   0:00 /bin/bash
johndoe        4  0.0  0.0  12720  8004 ?        R+   23:33   0:00 ps aux
[johndoe@hakoniwa data]$ exit
exit
[2025-04-03T15:33:06Z DEBUG] ================================
[2025-04-03T15:33:06Z DEBUG] Exited: Process(/bin/bash) exited with code 0
[2025-04-03T15:33:06Z DEBUG] Rusage: real time: 11.666743759s
[2025-04-03T15:33:06Z DEBUG] Rusage: user time: 20.56ms
[2025-04-03T15:33:06Z DEBUG] Rusage:  sys time: 12.183ms
```

More configuration files can be found in [hakoniwa.d](./hakoniwa.d).
