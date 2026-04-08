# --landlock-restrict-tcp-connect

Restrict network access rights for tcp connecting, this feature requires **minimum kernel version 6.7**.

```console
$ hakoniwa run --landlock-restrict-tcp-connect -- echo "OK"
OK

```

```console
$ hakoniwa run --landlock-restrict-tcp-connect -- aria2c https://example.com --async-dns-server=8.8.8.8 --dry-run --check-certificate=false
? 1
...
[..] Permission denied
...

```
