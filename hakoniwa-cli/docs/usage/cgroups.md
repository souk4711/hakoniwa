# Usage - Control Groups

## --cgroup-cpus

Specify how much of the available CPU resources a container can use

```console,ignore
$ hakoniwa run --cgroup-cpus 2 -- stress -c $(nproc --all)
stress: info: [1] dispatching hogs: 8 cpu, 0 io, 0 vm, 0 hdd
...
```

## --cgroup-memory

Specify the hard limit on memory usage in bytes

## --cgroup-memory-swap

Specify the hard limit on memory+swap usage in bytes

```console,ignore
$ hakoniwa run --cgroup-memory 512M --cgroup-memory-swap 512M -- stress --vm 4 --vm-bytes 256M
stress: info: [1] dispatching hogs: 0 cpu, 0 io, 4 vm, 0 hdd
stress: FAIL: [1] (425) <-- worker 5 got signal 9
stress: WARN: [1] (427) now reaping child worker processes
stress: FAIL: [1] (425) <-- worker 3 got signal 9
stress: WARN: [1] (427) now reaping child worker processes
stress: FAIL: [1] (425) <-- worker 4 got signal 9
stress: WARN: [1] (427) now reaping child worker processes
stress: FAIL: [1] (461) failed run completed in 0s
```

## --cgroup-pids-limit

Specify the maximum number of tasks

```console,ignore
$ hakoniwa run -w . --cgroup-pids-limit 2 -- /bin/python3 ./tests/fixtures/scripts/fork-bomb.py
Traceback (most recent call last):
  File "./tests/fixtures/scripts/fork-bomb.py", line 4, in <module>
    os.fork()
    ~~~~~~~^^
BlockingIOError: [Errno 11] Resource temporarily unavailable
```
