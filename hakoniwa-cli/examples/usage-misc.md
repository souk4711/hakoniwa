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
$ hakoniwa run --uidmap 0 -- id
uid=0(root) gid=65534(nobody) groups=65534(nobody)
```

## --gidmap

Custom GID in the container

```console
$ hakoniwa run --gidmap 0 -- id
uid=65534(nobody) gid=0(root) groups=0(root),65534(nobody)
```
