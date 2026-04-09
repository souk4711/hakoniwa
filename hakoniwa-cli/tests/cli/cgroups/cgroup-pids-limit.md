# --cgroup-pids-limit

Specifies the maximum number of tasks

## resource temporarily unavailable

```console
$ hakoniwa run -w . --cgroup-pids-limit 2 -- /bin/python3 ./tests/fixtures/scripts/fork-bomb.py
? 1
...
BlockingIOError: [Errno 11] Resource temporarily unavailable

```
