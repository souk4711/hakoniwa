# HowTo - Introduction to Config File

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
  { source = ""          , destination = "/dev"     , type = "devfs" },   # --devfs /dev
  { source = ""          , destination = "/tmp"     , type = "tmpfs" },   # --tmpfs /tmp
  { source = ""          , destination = "/run"     , type = "tmpfs" },   # --tmpfs /run
  { source = "{{ PWD }}" , destination = "{{ PWD }}",   rw = true    },   # --bindmount-rw $PWD
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
  { type = "as"          , value = 16000000 },  # --limit-as 16000000
  { type = "walltime"    , value = 60       },  # --limit-walltime 60
]

# seccomp
[seccomp]
path = "{{ __dir__ }}/abstractions/seccomp/audit.json"                # --seccomp ./examples/hakoniwa.d/abstractions/seccomp/audit.json

# cmdline
[command]
cmdline = ["bash"]
cwd = "{{ PWD }}"
```

Run:

```console,ignore
$ hakoniwa run -v -c ./examples/hakoniwa.d/example.toml
[2025-03-14T10:03:11Z DEBUG] CONFIG: ./examples/hakoniwa.d/example.toml
[2025-03-14T10:03:11Z DEBUG] Unshare namespaces: CloneFlags(CLONE_NEWNS | CLONE_NEWCGROUP | CLONE_NEWUTS | CLONE_NEWIPC | CLONE_NEWUSER | CLONE_NEWPID | CLONE_NEWNET)
[2025-03-14T10:03:11Z DEBUG] RootDir: "/tmp/hakoniwa-JO7Tne" -> "/"
[2025-03-14T10:03:11Z DEBUG] Mount: "/usr/bin" -> "/bin", FsType(), MsFlags(MS_RDONLY | MS_NOSUID | MS_BIND | MS_REC)
[2025-03-14T10:03:11Z DEBUG] Mount: "devfs" -> "/dev", FsType(devfs), MsFlags(0x0)
[2025-03-14T10:03:11Z DEBUG] Mount: "/etc" -> "/etc", FsType(), MsFlags(MS_RDONLY | MS_NOSUID | MS_BIND | MS_REC)
[2025-03-14T10:03:11Z DEBUG] Mount: "/home/johndoe/.../hakoniwa/hakoniwa-cli" -> "...", FsType(), MsFlags(MS_NOSUID | MS_BIND | MS_REC)
[2025-03-14T10:03:11Z DEBUG] Mount: "/usr/lib" -> "/lib", FsType(), MsFlags(MS_RDONLY | MS_NOSUID | MS_BIND | MS_REC)
[2025-03-14T10:03:11Z DEBUG] Mount: "/usr/lib" -> "/lib64", FsType(), MsFlags(MS_RDONLY | MS_NOSUID | MS_BIND | MS_REC)
[2025-03-14T10:03:11Z DEBUG] Mount: "proc" -> "/proc", FsType(proc), MsFlags(MS_NOSUID | MS_NODEV | MS_NOEXEC)
[2025-03-14T10:03:11Z DEBUG] Mount: "tmpfs" -> "/run", FsType(tmpfs), MsFlags(MS_NOSUID | MS_NODEV)
[2025-03-14T10:03:11Z DEBUG] Mount: "/usr/bin" -> "/sbin", FsType(), MsFlags(MS_RDONLY | MS_NOSUID | MS_BIND | MS_REC)
[2025-03-14T10:03:11Z DEBUG] Mount: "tmpfs" -> "/tmp", FsType(tmpfs), MsFlags(MS_NOSUID | MS_NODEV)
[2025-03-14T10:03:11Z DEBUG] Mount: "/usr" -> "/usr", FsType(), MsFlags(MS_RDONLY | MS_NOSUID | MS_BIND | MS_REC)
[2025-03-14T10:03:11Z DEBUG] UID mapping: container_id: 1000, host_id: 1000, count: 1
[2025-03-14T10:03:11Z DEBUG] GID mapping: container_id: 1000, host_id: 1000, count: 1
[2025-03-14T10:03:11Z DEBUG] Env: LANGUAGE=en_US
[2025-03-14T10:03:11Z DEBUG] Env: TERM=xterm-256color
[2025-03-14T10:03:11Z DEBUG] Env: VAR123=456
[2025-03-14T10:03:11Z DEBUG] Env: LANG=en_US.UTF-8
[2025-03-14T10:03:11Z DEBUG] Seccomp: Load 1 rules for architectures()
[2025-03-14T10:03:11Z DEBUG] Execve: "/usr/bin/bash", []
[johndoe@hakoniwa hakoniwa-cli]$ ls
Cargo.toml  examples  LICENSE  src  tests
[johndoe@hakoniwa hakoniwa-cli]$ env
LANGUAGE=en_US
PWD=...
LANG=en_US.UTF-8
VAR123=456
TERM=xterm-256color
SHLVL=1
_=/usr/bin/env
[johndoe@hakoniwa hakoniwa-cli]$ ps aux
USER         PID %CPU %MEM    VSZ   RSS TTY      STAT START   TIME COMMAND
johndoe        1  0.1  0.0  12188  9448 ?        S    15:45   0:00 /usr/bin/bash
johndoe        3  0.0  0.0  12720  7964 ?        R+   15:45   0:00 ps aux
[johndoe@hakoniwa hakoniwa-cli]$
...
[2025-03-14T07:46:25Z DEBUG] Exited: Process(/usr/bin/bash) received signal SIGKILL
[2025-03-14T07:46:25Z DEBUG] Rusage: real time: 60.000650929s
[2025-03-14T07:46:25Z DEBUG] Rusage: user time: 20.134ms
[2025-03-14T07:46:25Z DEBUG] Rusage:  sys time: 13.607ms
[2025-03-14T07:46:25Z ERROR] hakoniwa: Process(/usr/bin/bash) received signal SIGKIL
```

More configuration files can be found in [hakoniwa.d](./hakoniwa.d).
