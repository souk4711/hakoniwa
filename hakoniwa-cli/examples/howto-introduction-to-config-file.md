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
path = "{{ __dir__ }}/abstractions/seccomp/fine-grained.json"             # --seccomp ./examples/hakoniwa.d/abstractions/seccomp/fine-grained.json

# cmdline
[command]
cmdline = ["/bin/bash"]
cwd = "/data"
```

Run:

```console,ignore
$ hakoniwa run -v -c ./examples/hakoniwa.d/example.toml
[2025-03-27T20:07:13Z DEBUG] CONFIG: ./examples/hakoniwa.d/example.toml
[2025-03-27T20:07:13Z DEBUG] Unshare namespaces: CloneFlags(CLONE_NEWNS | CLONE_NEWCGROUP | CLONE_NEWUTS | CLONE_NEWIPC | CLONE_NEWUSER | CLONE_NEWPID | CLONE_NEWNET)
[2025-03-27T20:07:13Z DEBUG] RootDir: "/tmp/hakoniwa-zjOG8f" -> "/"
[2025-03-27T20:07:13Z DEBUG] Mount: "/usr/bin" -> "/bin", FsType(), MsFlags(MS_RDONLY | MS_NOSUID | MS_BIND | MS_REC)
[2025-03-27T20:07:13Z DEBUG] Mount: "devfs" -> "/dev", FsType(devfs), MsFlags(0x0)
[2025-03-27T20:07:13Z DEBUG] Mount: "/etc" -> "/etc", FsType(), MsFlags(MS_RDONLY | MS_NOSUID | MS_BIND | MS_REC)
[2025-03-27T20:07:13Z DEBUG] Mount: "/home/johndoe/..." -> "/data", FsType(), MsFlags(MS_NOSUID | MS_BIND | MS_REC)
[2025-03-27T20:07:13Z DEBUG] Mount: "/usr/lib" -> "/lib", FsType(), MsFlags(MS_RDONLY | MS_NOSUID | MS_BIND | MS_REC)
[2025-03-27T20:07:13Z DEBUG] Mount: "/usr/lib" -> "/lib64", FsType(), MsFlags(MS_RDONLY | MS_NOSUID | MS_BIND | MS_REC)
[2025-03-27T20:07:13Z DEBUG] Mount: "proc" -> "/proc", FsType(proc), MsFlags(MS_NOSUID | MS_NODEV | MS_NOEXEC)
[2025-03-27T20:07:13Z DEBUG] Mount: "tmpfs" -> "/run", FsType(tmpfs), MsFlags(MS_NOSUID | MS_NODEV)
[2025-03-27T20:07:13Z DEBUG] Mount: "/usr/bin" -> "/sbin", FsType(), MsFlags(MS_RDONLY | MS_NOSUID | MS_BIND | MS_REC)
[2025-03-27T20:07:13Z DEBUG] Mount: "tmpfs" -> "/tmp", FsType(tmpfs), MsFlags(MS_NOSUID | MS_NODEV)
[2025-03-27T20:07:13Z DEBUG] Mount: "/usr" -> "/usr", FsType(), MsFlags(MS_RDONLY | MS_NOSUID | MS_BIND | MS_REC)
[2025-03-27T20:07:13Z DEBUG] UID mapping: container_id: 1000, host_id: 1000, count: 1
[2025-03-27T20:07:13Z DEBUG] GID mapping: container_id: 1000, host_id: 1000, count: 1
[2025-03-27T20:07:13Z DEBUG] Env: TERM=xterm-256color
[2025-03-27T20:07:13Z DEBUG] Env: VAR123=456
[2025-03-27T20:07:13Z DEBUG] Env: LANG=en_US.UTF-8
[2025-03-27T20:07:13Z DEBUG] Env: LANGUAGE=en_US
[2025-03-27T20:07:13Z DEBUG] Seccomp: Load 372 rules for architectures(X86, X32, X8664)
[2025-03-27T20:07:13Z DEBUG] Execve: "/bin/bash", []
[2025-03-27T20:07:13Z DEBUG] ================================
[johndoe@hakoniwa hakoniwa]$ ls
Cargo.lock  Cargo.toml  CODE_OF_CONDUCT.md  deny.toml  docs  hakoniwa  hakoniwa-cli  LICENSE.md  README.md  scripts  target
[johndoe@hakoniwa hakoniwa]$ env
LANGUAGE=en_US
PWD=/data
LANG=en_US.UTF-8
VAR123=456
TERM=xterm-256color
SHLVL=1
_=/usr/bin/env
[johndoe@hakoniwa hakoniwa]$ ps aux
USER         PID %CPU %MEM    VSZ   RSS TTY      STAT START   TIME COMMAND
johndoe        1  0.0  0.0  12192  9460 ?        S    04:07   0:00 /bin/bash
johndoe        4  0.0  0.0  12720  7852 ?        R+   04:07   0:00 ps aux
[johndoe@hakoniwa hakoniwa]$ exit
exit
[2025-03-27T20:07:45Z DEBUG] ================================
[2025-03-27T20:07:45Z DEBUG] Exited: Process(/bin/bash) exited with code 0
[2025-03-27T20:07:45Z DEBUG] Rusage: real time: 31.554767084s
[2025-03-27T20:07:45Z DEBUG] Rusage: user time: 18.001ms
[2025-03-27T20:07:45Z DEBUG] Rusage:  sys time: 15.215ms
[2025-03-14T07:46:25Z ERROR] hakoniwa: Process(/bin/bash) received signal SIGKIL
```

More configuration files can be found in [hakoniwa.d](./hakoniwa.d).
