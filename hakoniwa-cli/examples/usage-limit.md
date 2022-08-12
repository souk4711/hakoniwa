# Usage - Process Resource Limit

## --limit-as

Limit the maximum size of the COMMAND's virtual memory

```sh
# 16MB
$ hakoniwa run --limit-as 16000000 -- stress --vm 1 --vm-bytes 16M
stress: info: [1] dispatching hogs: 0 cpu, 0 io, 1 vm, 0 hdd
stress: FAIL: [2] (495) hogvm malloc failed: Cannot allocate memory
stress: FAIL: [1] (395) <-- worker 2 returned error 1
stress: WARN: [1] (397) now reaping child worker processes
stress: FAIL: [1] (401) kill error: No such process
stress: FAIL: [1] (452) failed run completed in 0s
```

## --limit-core

Limit the maximum size of a core file in bytes that the COMMAND may dump

```sh
# No core file
$ hakoniwa run --limit-core 0 -- echo
```

## --limit-cpu

Limit the amount of CPU time that the COMMAND can consume, in seconds

```sh
# Killed in 2s
$ hakoniwa run --limit-cpu 2 -- stress -c 1
stress: info: [1] dispatching hogs: 1 cpu, 0 io, 0 vm, 0 hdd
stress: FAIL: [1] (416) <-- worker 2 got signal 9
stress: WARN: [1] (418) now reaping child worker processes
stress: FAIL: [1] (422) kill error: No such process
stress: FAIL: [1] (452) failed run completed in 2s

# Not killed in 2s, see also '--limit-walltime'
$ date; hakoniwa run --limit-cpu 2 -- sleep 5; date
Fri Aug 12 09:17:04 AM UTC 2022
Fri Aug 12 09:17:09 AM UTC 2022
```

## --limit-fsize

Limit the maximum size in bytes of files that the COMMAND may create

```sh
# 2bytes
$ hakoniwa run --limit-fsize 2 -- echo "abcd" > output.txt
echo: write error: File too large
```

## --limit-nofile

Limit the maximum file descriptor number that can be opened by the COMMAND

```sh
# 2
$ hakoniwa run --limit-nofile 2 -- echo
echo: error while loading shared libraries: libc.so.6: cannot open shared object file: Error 24
```

## --limit-walltime

Limit the amount of wall time that the COMMAND can consume, in seconds

```sh
# Killed in 2s
$ date; hakoniwa run --limit-walltime 2 -- sleep 5; date
Fri Aug 12 09:17:40 AM UTC 2022
Fri Aug 12 09:17:42 AM UTC 2022
```
