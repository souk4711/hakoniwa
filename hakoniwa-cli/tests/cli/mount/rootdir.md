# --rootdir

Use ROOTDIR as the mount point for the container root fs

## with RO options

```console
$ hakoniwa run --rootfs=none --rootdir ./tests/fixtures/rootfs -- /bin/cat /proc/1/mountinfo
[..]/rootfs / ro,relatime[..]
...
```

## with RW options

```console
$ hakoniwa run --rootfs=none --rootdir ./tests/fixtures/rootfs:rw -- /bin/cat /proc/1/mountinfo
[..]/rootfs / rw,relatime[..]
...
```
