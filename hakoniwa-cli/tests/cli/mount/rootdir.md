# --rootdir

Use ROOTDIR as the mount point for the container root fs

## with RO options

```console
$ hakoniwa run --rootdir ../hakoniwa/tests/fixtures/alpine-x86_64    -- /bin/cat /proc/1/mountinfo
[..]/alpine-x86_64 / ro,relatime[..]
...
```

## with RW options

```console
$ hakoniwa run --rootdir ../hakoniwa/tests/fixtures/alpine-x86_64:rw -- /bin/cat /proc/1/mountinfo
[..]/alpine-x86_64 / rw,relatime[..]
...
```
