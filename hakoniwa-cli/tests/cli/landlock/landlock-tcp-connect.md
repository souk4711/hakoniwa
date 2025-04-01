# --landlock-tcp-connect

Allow connecting an active TCP socket to a remote port

## allow

```console
$ hakoniwa run --landlock-tcp-connect 443 -- aria2c https://example.com --dry-run
...
(OK):download completed.

```

## block


```console
$ hakoniwa run --landlock-tcp-connect 442 -- aria2c https://example.com --dry-run
? 1
...
[..] Permission denied
...

```
