# Usage - Misc

## --hostname

Custom hostname in the container (implies --unshare-uts)

```console
$ hakoniwa run --hostname myhost -- hostname
myhost

```

## --uidmap

Custom UID in the container

```console,ignore
$ hakoniwa run --uidmap 0 -- id
uid=0(root) gid=1000(johndoe) groups=1000(johndoe),65534(nobody)

```

## --gidmap

Custom GID in the container

```console,ignore
$ hakoniwa run --gidmap 0 -- id
uid=1000(johndoe) gid=0(root) groups=0(root),65534(nobody)

```
