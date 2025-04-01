# --landlock-fs-rw

Allow to read-write files beneath PATH

## read file

```console
$ hakoniwa run --landlock-fs-rx /bin:/lib --landlock-fs-rw /etc -- cat /etc/hosts
...

```

## write file

```console
$ hakoniwa run --tmpfs /tmp --landlock-fs-rx /bin:/lib --landlock-fs-rw /tmp -- touch /tmp/myfile.txt

```

## cannot execute file

```console
$ hakoniwa run --tmpfs /tmp --landlock-fs-rx /bin:/lib --landlock-fs-rw /tmp -- sh -c "cp /bin/echo /tmp/echo && /tmp/echo"
? 126
[..] Permission denied

```
