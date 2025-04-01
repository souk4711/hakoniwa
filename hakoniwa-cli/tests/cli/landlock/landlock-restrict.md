# --landlock-restrict

Restrict ambient rights (e.g. global filesystem access) for the process

## fs

```console
$ hakoniwa run --landlock-restrict=fs -- echo "OK"
? 125
[..] Permission denied

```

## net.tcp.bind

```console
$ hakoniwa run -w . --landlock-restrict=net.tcp.bind -- /bin/python3 ./tests/fixtures/scripts/httpd-1s.py
? 1
...
[..] Permission denied
...

$ hakoniwa run -w . --landlock-restrict=tcp.bind -- /bin/python3 ./tests/fixtures/scripts/httpd-1s.py
? 1
...
[..] Permission denied
...

$ hakoniwa run -w . --landlock-restrict=tcp.bind -- echo "OK"
OK

```

## net.tcp.connect

```console
$ hakoniwa run --landlock-restrict=net.tcp.connect -- aria2c https://example.com --async-dns-server=8.8.8.8 --dry-run
? 1
...
[..] Permission denied
...

$ hakoniwa run --landlock-restrict=tcp.connect -- aria2c https://example.com --async-dns-server=8.8.8.8 --dry-run
? 1
...
[..] Permission denied
...

$ hakoniwa run --landlock-restrict=tcp.connect -- echo "OK"
OK

```
