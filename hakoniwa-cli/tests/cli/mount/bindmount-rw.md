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
