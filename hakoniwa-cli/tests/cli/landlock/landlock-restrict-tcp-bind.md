# --landlock-restrict-tcp-bind

Restrict network access rights for tcp binding, this feature requires **minimum kernel version 6.7**.

```console
$ hakoniwa run -w . --landlock-restrict-tcp-bind -- echo "OK"
OK

```

```console
$ hakoniwa run -w . --landlock-restrict-tcp-bind -- /bin/python3 ./tests/fixtures/scripts/httpd-1s.py
? 1
...
[..] Permission denied
...
```
