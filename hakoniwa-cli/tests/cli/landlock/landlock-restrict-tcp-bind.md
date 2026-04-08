# --landlock-restrict-tcp-bind

Restrict network access rights for tcp binding

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
