# --landlock-restrict-all

Restrict ambient rights (e.g. global filesystem access) for the process

```console
$ hakoniwa run --landlock-restrict-all -- echo "OK"
? 125
[..] Permission denied

```
