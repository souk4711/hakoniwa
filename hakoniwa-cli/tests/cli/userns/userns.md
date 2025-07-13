# --userns

Configure user namespace for the container

### auto

```console
$ hakoniwa run --userns=auto -- cat /proc/self/uid_map
         0    [..]          1
         1    [..]

```

```console
$ hakoniwa run --userns=auto -- cat /proc/self/gid_map
         0    [..]          1
         1    [..]

```
