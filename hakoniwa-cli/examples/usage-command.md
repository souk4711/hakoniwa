# Usage - COMMAND

## --setenv

Set an environment variable

```console
$ hakoniwa run --setenv SHELL=/bin/bash -- env
SHELL=/bin/bash

```

## --workdir

Bind mount the HOST_PATH on "/hako" with read-write access, then run COMMAND in "/hako"

```console,ignore
# Bindmount, then change to "/hako"
$ hakoniwa run --workdir $PWD -- pwd
/hako

# Or change to a designated CONTAINER_PATH
$ hakoniwa run --bindmount $PWD:/mytmp --workdir :/mytmp -- pwd
/mytmp
```
