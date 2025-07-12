# --uidmap

UID map to use for the user namespace (repeatable)

## mapping UID

```console
$ hakoniwa run --uidmap 0 -- id
uid=0(root) gid=1[..]

```

## cli arg name `-u`

```console
$ hakoniwa run -u 0 -- id
uid=0(root) gid=1[..]

```
