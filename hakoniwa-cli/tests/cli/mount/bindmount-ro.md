# --bindmount-ro

Bind mount the HOST_PATH on CONTAINER_PATH with read-only access

## mount options contains `ro`

```console
$ hakoniwa run --bindmount-ro .:/mytmp -- findmnt /mytmp
TARGET [..] OPTIONS
/mytmp [..] ro,[..]

```

## cannot write file

```console
$ hakoniwa run --bindmount-ro .:/mytmp -- touch /mytmp/Cargo.toml
? 1
[..] Read-only file system

```

## cli arg name `-b`

```console
$ hakoniwa run -b .:/mytmp -- findmnt /mytmp
TARGET [..] OPTIONS
/mytmp [..] ro,[..]

```

## cli arg value `HOST_PATH:CONTAINER_PATH`

```console
$ hakoniwa run -b /home:/myhome -- ls /
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
$ hakoniwa run -b /home -- ls /
bin
etc
home
lib
lib64
proc
sbin
usr

```
