# Usage - COMMAND

## --setenv

Set an environment variable

```console
$ hakoniwa run --setenv ENV1=abc -- env
ENV1=abc
```

## --workdir

Bind mount the HOST_PATH on "/hako" with read-write access, then run COMMAND in "/hako"

```console
$ # Bind mount $PWD on "/hako", then change to "/hako"
$ hakoniwa run --workdir . -- pwd
/hako

$ # Or change to a designated CONTAINER_PATH
$ hakoniwa run --bindmount .:/mytmp --workdir :/mytmp -- pwd
/mytmp
```
