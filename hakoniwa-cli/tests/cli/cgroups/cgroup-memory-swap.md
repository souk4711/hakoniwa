# --cgroup-memory-swap

Specify the hard limit on memory+swap usage in bytes

## Out of memory

```console
$ hakoniwa run --cgroup-memory 1M --cgroup-memory-swap 1M -- true
? 137
hakoniwa: process(/usr/bin/true) received signal SIGKILL

```
