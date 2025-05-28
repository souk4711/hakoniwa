# HowTo - Create Profile

## Example

Create a configuration file:

```toml
# ./profiles/example.toml

# constants
{% set home = os_env("HOME") %}
{% set pwd  = os_env("PWD")  %}

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
  { source = "{{ pwd }}" , destination = "/data",   rw = true    },   # --bindmount-rw $PWD
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
path = "{{ __dir__ }}/seccomp/fine-grained.json"

# cmdline
[command]
cmdline = ["/usr/bin/bash"]
cwd = "/data"
```

Run:

```console
$ hakoniwa run -v -c ./profiles/example.toml
[2025-04-03T15:32:55Z DEBUG] CONFIG: ./profiles/example.toml
[2025-04-03T15:32:55Z DEBUG] Unshare namespaces: CloneFlags(CLONE_NEWNS | CLONE_NEWCGROUP | CLONE_NEWUTS | CLONE_NEWIPC | CLONE_NEWUSER | CLONE_NEWPID | CLONE_NEWNET)
[2025-04-03T15:32:55Z DEBUG] RootDir: "/tmp/hakoniwa-6Vy9RB" -> "/"
...
```

More configuration files can be found in [profiles](./profiles).
