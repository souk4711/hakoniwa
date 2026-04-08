# --landlock-restrict-tcp-connect

Restrict network access rights for tcp connecting

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
