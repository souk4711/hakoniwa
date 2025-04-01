# --landlock-fs-rx

Allow to execute files beneath PATH (implies --landlock-restrict=fs)

## read file

```console
$ hakoniwa run --landlock-fs-rx /bin:/lib:/etc -- cat /etc/hosts
...

```

## cannot write file

```console
$ hakoniwa run --tmpfs /tmp --landlock-fs-rx /bin:/lib:/tmp -- touch /tmp/myfile.txt
? 1
[..] Permission denied

```
