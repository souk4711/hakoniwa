# Usage - User

## --userns

Configure user namespace for the container

> [!NOTE]
> This option conflicts with `--uidmap` and `--gidmap`.

### auto

Map current user/group to root, and use all the subuids/subgids.

```console,ignore
$ hakoniwa run --userns=auto -- cat /proc/self/uid_map
         0       1000          1
         1     100000      65536

$ hakoniwa run --userns=auto -- cat /proc/self/gid_map
         0       1000          1
         1     100000      65536
```

## --uidmap (alias -u)

UID map to use for the user namespace (repeatable)

> [!NOTE]
> This option conflicts with `--userns`.

```console,ignore
$ # Map current user to itself
$ hakoniwa run -- cat /proc/self/uid_map
      1000       1000          1

$ # Map current user to root
$ hakoniwa run --uidmap 0 -- cat /proc/self/uid_map
         0       1000          1

$ # Map current user to root, and use all the subuids
$ hakoniwa run --uidmap 0 --uidmap 1:100000:65536 -- cat /proc/self/uid_map
         0       1000          1
         1     100000      65536
```

## --gidmap (alias -g)

GID map to use for the user namespace (repeatable)

> [!NOTE]
> This option conflicts with `--userns`.

```console,ignore
$ # Map current group to itself
$ hakoniwa run -- cat /proc/self/gid_map
      1000       1000          1

$ # Map current group to root
$ hakoniwa run --gidmap 0 -- cat /proc/self/gid_map
         0       1000          1

$ # Map current group to root, and use all the subgids
$ hakoniwa run --gidmap 0 --gidmap 1:100000:65536 -- cat /proc/self/gid_map
         0       1000          1
         1     100000      65536
```
