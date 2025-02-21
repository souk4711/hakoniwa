# --workdir

Bind mount the HOST_PATH on "/hako" with read-write access, then run COMMAND in "/hako"

## Bindmount, then change to "/hako"

```console
$ hakoniwa run --workdir . -- pwd
/hako

```

## Change to a designated CONTAINER_PATH

```console
$ hakoniwa run --bindmount .:/mytmp --workdir :/mytmp -- pwd
/mytmp

```
