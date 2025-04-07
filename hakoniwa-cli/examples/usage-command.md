# Usage - COMMAND

## --setenv (alias -e)

Set an environment variable (repeatable)

```console
$ hakoniwa run --setenv ENV1=abc -- env
ENV1=abc

```

## --workdir (alias -w)

Bind mount the HOST_PATH on the same container path with read-write access, then run COMMAND inside it

```console
$ hakoniwa run --workdir . -- pwd
[CWD]

$ hakoniwa run --bindmount-ro .:/mytmp --workdir :/mytmp -- pwd
/mytmp

```
