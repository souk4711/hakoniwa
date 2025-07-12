# --workdir

Bind mount the HOST_PATH on the same container path with read-write access, then run COMMAND inside it

## bind mount $PWD

```console
$ hakoniwa run --workdir . -- pwd
[CWD]

```

## change to a designated CONTAINER_PATH

```console
$ hakoniwa run --bindmount-rw .:/mytmp --workdir :/mytmp -- pwd
/mytmp

```

## cli arg name `-w`

```console
$ hakoniwa run -w . -- pwd
[CWD]

```
