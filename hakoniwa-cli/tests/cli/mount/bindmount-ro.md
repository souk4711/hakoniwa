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
