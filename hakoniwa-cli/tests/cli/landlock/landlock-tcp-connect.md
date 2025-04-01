# --landlock-tcp-connect

Allow connecting an active TCP socket to a remote port (implies --landlock-restrict=tcp.bind)

## allow

```console
$ hakoniwa run --landlock-tcp-connect 443 -- aria2c https://example.com --async-dns-server=8.8.8.8 --dry-run
...
(OK):download completed.

```

## unallowed

```console
$ hakoniwa run --landlock-tcp-connect 442 -- aria2c https://example.com --async-dns-server=8.8.8.8 --dry-run
? 1
...
[..] Permission denied
...

```
