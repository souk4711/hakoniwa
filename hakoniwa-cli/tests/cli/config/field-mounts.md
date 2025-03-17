# CfgMount

## CfgMount#destination `NULL`

```console
$ hakoniwa run --config ./tests/fixtures/config/field-mounts.toml -- findmnt /sys
TARGET [..] OPTIONS
/sys [..] rw,[..]

```

## CfgMount#rw `false`

```console
$ hakoniwa run --config ./tests/fixtures/config/field-mounts.toml -- findmnt /rodir
TARGET [..] OPTIONS
/rodir [..] ro,[..]

```

## CfgMount#rw `true`

```console
$ hakoniwa run --config ./tests/fixtures/config/field-mounts.toml -- findmnt /rwdir
TARGET [..] OPTIONS
/rwdir [..] rw,[..]

```

## CfgMount#type `devfs`

```console
$ hakoniwa run --config ./tests/fixtures/config/field-mounts.toml -- dd if=/mydev/random of=/mytmp/output.txt count=1 bs=4
1+0 records in
1+0 records out
4 bytes copied, [..]

```

## CfgMount#type `tmpfs`

```console
$ hakoniwa run --config ./tests/fixtures/config/field-mounts.toml -- findmnt /mytmp
TARGET [..] OPTIONS
/mytmp [..] rw,nosuid,nodev,[..]

```
