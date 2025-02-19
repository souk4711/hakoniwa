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

## --workdir

Bind mount the HOST_PATH on "/hako" with read-write access, then run COMMAND in "/hako"

```console
$ touch myfile.txt
$ hakoniwa run --workdir $PWD -- ls
myfile.txt

# Change to a designated CONTAINER_PATH
$ hakoniwa run --bindmount $PWD:/mytmp --workdir :/mytmp -- pwd
/mytmp
```
