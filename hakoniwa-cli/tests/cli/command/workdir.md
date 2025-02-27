# --workdir

Bind mount the HOST_PATH on "/hako" with read-write access, then run COMMAND in "/hako"

## Bind mount $PWD on "/hako", then change to "/hako"

```console
$ hakoniwa run --workdir . -- pwd
/hako

```

## Change to a designated CONTAINER_PATH

```console
$ hakoniwa run --bindmount-rw .:/mytmp --workdir :/mytmp -- pwd
/mytmp

```

## cli arg name `-w`

```console
$ hakoniwa run -w . -- pwd
/hako

```
