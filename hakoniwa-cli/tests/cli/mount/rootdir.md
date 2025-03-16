# --rootdir

Use ROOTDIR as the mount point for the container root fs

## with RO options

```console
$ hakoniwa run --rootdir ../hakoniwa/tests/rootfs --rootfs /var/empty -- /bin/cat /proc/1/mountinfo
[..]/rootfs / ro,relatime[..]
...
```

## with RW options

```console
$ hakoniwa run --rootdir ../hakoniwa/tests/rootfs:rw --rootfs /var/empty -- /bin/cat /proc/1/mountinfo
[..]/rootfs / rw,relatime[..]
...
```
