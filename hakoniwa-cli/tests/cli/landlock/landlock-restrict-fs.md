# --landlock-restrict-fs

Restrict access rights to the entire file system

## restrict access rights

```console
$ hakoniwa run --landlock-restrict-fs -- echo
? 125
[..] Permission denied

$ hakoniwa run --landlock-fs-rx /bin:/lib -- echo


$ hakoniwa run --landlock-fs-rx /bin,/lib -- echo


$ hakoniwa run --landlock-fs-rx /bin,/lib -- ls /bin/ls
/bin/ls

$ hakoniwa run --landlock-fs-rx /bin,/lib -- ls /etc
? 2
[..] Permission denied

```
