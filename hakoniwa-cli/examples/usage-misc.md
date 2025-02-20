# Usage - Misc

## --hostname

Custom hostname in the container (implies --unshare-uts)

```console
$ hakoniwa run --hostname myhost -- hostname
myhost

```

## --uidmap

Custom UID in the container

```console
$ hakoniwa run --uidmap 0 -- id -u
0

```

## --gidmap

Custom GID in the container

```console
$ hakoniwa run --gidmap 0 -- id -g
0

```
