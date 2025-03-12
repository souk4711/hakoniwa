# --bindmount-rw

Bind mount the HOST_PATH on CONTAINER_PATH with read-write access

## mount options contains `rw`

```console
$ hakoniwa run --bindmount-rw .:/mytmp -- findmnt /mytmp
TARGET [..] OPTIONS
/mytmp [..] rw,[..]

```

## can write file

```console
$ hakoniwa run --bindmount-rw .:/mytmp -- touch /mytmp/Cargo.toml
? 0
```

## cli arg name `-B`

```console
$ hakoniwa run -B .:/mytmp -- findmnt /mytmp
TARGET [..] OPTIONS
/mytmp [..] rw,[..]

```

## cli arg value `HOST_PATH:CONTAINER_PATH`

```console
$ hakoniwa run -B /home:/myhome -- ls /
bin
etc
lib
lib64
myhome
proc
sbin
usr

```

## cli arg value `HOST_PATH`

```console
$ hakoniwa run -B /home -- ls /
bin
etc
home
lib
lib64
proc
sbin
usr

```
