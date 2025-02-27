# --setenv

Set an environment variable

## setenv

```console
$ hakoniwa run --setenv DISPLAY=:1 -- env
DISPLAY=:1

```

## cli arg name `-e`

```console
$ hakoniwa run -e DISPLAY=:1 -- env
DISPLAY=:1

```

## cli arg value `NAME=VALUE`

```console
$ hakoniwa run --setenv SESSION_MANAGER=local/archlinux:@/tmp/.ICE-unix/1400,unix/archlinux:/tmp/.ICE-unix/1400 -- env
SESSION_MANAGER=local/archlinux:@/tmp/.ICE-unix/1400,unix/archlinux:/tmp/.ICE-unix/1400

```

## cli arg value `NAME:VALUE`

```console
$ hakoniwa run --setenv SESSION_MANAGER:local/archlinux:@/tmp/.ICE-unix/1400,unix/archlinux:/tmp/.ICE-unix/1400 -- env
SESSION_MANAGER=local/archlinux:@/tmp/.ICE-unix/1400,unix/archlinux:/tmp/.ICE-unix/1400

```

## cli arg value `NAME`

```console
$ XDG_SESSION_ID=1 hakoniwa run --setenv XDG_SESSION_ID -- env
XDG_SESSION_ID=1

```
