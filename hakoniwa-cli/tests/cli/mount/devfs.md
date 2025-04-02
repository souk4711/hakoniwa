# --devfs

Mount new devfs on CONTAINER_PATH (repeatable)

## mount options contains `rw,`

```console
$ hakoniwa run --devfs /mydev -- findmnt /mydev
TARGET [..] OPTIONS
/mydev [..] rw,[..]

```

## can read `/dev/random`

```console
$ hakoniwa run --devfs /mydev --tmpfs /mytmp -- dd if=/mydev/random of=/mytmp/output.txt count=1 bs=4
1+0 records in
1+0 records out
4 bytes copied, [..]
```

## can write `/dev/null`

```console
$ hakoniwa run --devfs /mydev -- sh -c "echo 'abc' > /mydev/null"
```
