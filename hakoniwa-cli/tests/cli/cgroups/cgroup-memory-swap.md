# --cgroup-memory-swap

Specify the hard limit on memory+swap usage in bytes

## Out of memory

```console
$ hakoniwa run --devfs /dev --tmpfs /tmp --cgroup-memory 32M --cgroup-memory-swap 32M -- dd if=/dev/random of=/tmp/output.txt count=1 bs=64M
? 137
hakoniwa: process(/usr/bin/dd) received signal SIGKILL

```
