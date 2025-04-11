# Usage - Process Resource Limit

## --limit-as

Limit the maximum size of the COMMAND's virtual memory

```console,ignore
$ hakoniwa run --limit-as 16000000 -- stress --vm 1 --vm-bytes 16M
stress: info: [1] dispatching hogs: 0 cpu, 0 io, 1 vm, 0 hdd
stress: FAIL: [2] (512) hogvm malloc failed: Cannot allocate memory
stress: FAIL: [1] (404) <-- worker 2 returned error 1
stress: WARN: [1] (406) now reaping child worker processes
stress: FAIL: [1] (410) kill error: No such process
stress: FAIL: [1] (461) failed run completed in 0s
```

## --limit-core

Limit the maximum size of a core file in bytes that the COMMAND may dump

## --limit-cpu

Limit the amount of CPU time that the COMMAND can consume, in seconds

```console,ignore
$ # Killed in 2s
$ hakoniwa run --limit-cpu 2 -- stress -c 1
stress: info: [1] dispatching hogs: 1 cpu, 0 io, 0 vm, 0 hdd
stress: FAIL: [1] (425) <-- worker 2 got signal 9
stress: WARN: [1] (427) now reaping child worker processes
stress: FAIL: [1] (431) kill error: No such process
stress: FAIL: [1] (461) failed run completed in 2s

$ # Not killed in 2s, see also `--limit-walltime`
$ date; hakoniwa run --limit-cpu 2 -- sleep 5; date
Wed Feb 19 04:18:54 PM HKT 2025
Wed Feb 19 04:18:59 PM HKT 2025
```

## --limit-fsize

Limit the maximum size in bytes of files that the COMMAND may create

```console,ignore
$ hakoniwa run --devfs /dev --tmpfs /tmp --limit-fsize 2 -- dd if=/dev/random of=/tmp/output.txt count=1 bs=4
dd: error writing '/tmp/output.txt': File too large
1+0 records in
0+0 records out
2 bytes copied, 2.5127e-05 s, 79.6 kB/s
```

## --limit-nofile

Limit the maximum file descriptor number that can be opened by the COMMAND

```console,ignore
$ hakoniwa run --limit-nofile 2 -- echo
/usr/bin/echo: error while loading shared libraries: libc.so.6: cannot open shared object file: Error 24
```

## --limit-walltime

Limit the amount of wall time that the COMMAND can consume, in seconds

```console,ignore
$ date; hakoniwa run --limit-walltime 2 -- sleep 5; date
Fri Feb 21 05:51:22 PM HKT 2025
hakoniwa: Process(/usr/bin/sleep) received signal SIGKILL
Fri Feb 21 05:51:24 PM HKT 2025
```
