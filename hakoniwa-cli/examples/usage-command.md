# Usage - COMMAND

## --setenv

Set an environment variable

```console
$ hakoniwa run --setenv ENV1=abc -- env
ENV1=abc

```

## --workdir

Bind mount the HOST_PATH on the same container path with read-write access, then run COMMAND inside it

```console
$ hakoniwa run --workdir . -- pwd
[CWD]

$ hakoniwa run --bindmount-rw .:/mytmp --workdir :/mytmp -- pwd
/mytmp

```
