# --landlock-restrict-fs

Restrict filesystem access rights, this feature requires **minimum kernel version 5.13**.

```console
$ hakoniwa run --landlock-restrict-fs -- echo "OK"
? 125
[..] Permission denied

```
