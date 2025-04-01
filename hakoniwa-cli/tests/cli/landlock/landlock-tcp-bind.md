# --landlock-tcp-bind

Allow binding a TCP socket to a local port (implies --landlock-restrict=tcp.bind)

## allow

```console
$ hakoniwa run -w . --landlock-tcp-bind 8000 -- /bin/python3 ./tests/fixtures/scripts/httpd-1s.py
Serving on port 8000
Signal handler called with signal 14
Shutdown...

```

## unallowed

```console
$ hakoniwa run -w . --landlock-tcp-bind 7999 -- /bin/python3 ./tests/fixtures/scripts/httpd-1s.py
? 1
...
[..] Permission denied
...

```
