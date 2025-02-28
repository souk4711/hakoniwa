# Usage - Mount FileSystem

## --rootdir

Use HOST_PATH as the mount point for the container root fs

## --rootfs

Bind mount all necessary subdirectories in ROOTFS to the container root with read-only access [default: /]

```console,ignore
$ mkdir -p rootfs && docker export $(docker create alpine) | tar -C rootfs -xf - && rmdir rootfs/proc
$ hakoniwa run --rootfs rootfs -- /bin/ls -l /bin
total 792
lrwxrwxrwx    1 nobody   nobody          12 Jan 26  2024 arch -> /bin/busybox
lrwxrwxrwx    1 nobody   nobody          12 Jan 26  2024 ash -> /bin/busybox
lrwxrwxrwx    1 nobody   nobody          12 Jan 26  2024 base64 -> /bin/busybox
lrwxrwxrwx    1 nobody   nobody          12 Jan 26  2024 bbconfig -> /bin/busybox
-rwxr-xr-x    1 nobody   nobody      808712 Nov  7  2023 busybox
lrwxrwxrwx    1 nobody   nobody          12 Jan 26  2024 cat -> /bin/busybox
lrwxrwxrwx    1 nobody   nobody          12 Jan 26  2024 chattr -> /bin/busybox
lrwxrwxrwx    1 nobody   nobody          12 Jan 26  2024 chgrp -> /bin/busybox
...
```

> [!NOTE]
> When use `/` as rootfs, it only mount following subdirectories: `/bin`, `/etc`, `/lib`, `/lib64`, `/sbin`, `/usr`.

## --bindmount-ro

Bind mount the HOST_PATH on CONTAINER_PATH with read-only access

```console,ignore
$ hakoniwa run --bindmount-ro .:/mytmp -- findmnt /mytmp
TARGET SOURCE                                           FSTYPE OPTIONS
/mytmp /dev/mapper/cryptroot[/home/johndoe/MyContainer] ext4   ro,relatime

$ hakoniwa run --bindmount-ro .:/mytmp -- touch /mytmp/myfile.txt
touch: cannot touch '/mytmp/myfile.txt': Read-only file system

```

## --bindmount-rw

Bind mount the HOST_PATH on CONTAINER_PATH with read-write access

```console,ignore
$ hakoniwa run --bindmount-rw .:/mytmp -- findmnt /mytmp
TARGET SOURCE                                           FSTYPE OPTIONS
/mytmp /dev/mapper/cryptroot[/home/johndoe/MyContainer] ext4   rw,relatime

$ hakoniwa run --bindmount-rw .:/mytmp -- touch /mytmp/myfile.txt
$ file myfile.txt
myfile.txt: empty
```

## --devfs

Mount new devfs on CONTAINER_PATH

```console,ignore
$ hakoniwa run --devfs /mydev -- ls -lah /mydev
total 0
drwxr-xr-x  4 johndoe johndoe    300 Feb 25 18:00 .
drwxr-xr-x 10 johndoe johndoe    200 Feb 25 18:00 ..
crw-------  1 johndoe nobody  136, 2 Feb 25 18:00 console
crw-rw-rw-  1 nobody  nobody    1, 7 Feb 24 15:43 full
crw-rw-rw-  1 nobody  nobody    1, 3 Feb 24 15:43 null
lrwxrwxrwx  1 johndoe johndoe     13 Feb 25 18:00 ptmx -> /mydev/pts/ptmx
drwxr-xr-x  2 nobody  nobody       0 Feb 25 18:00 pts
crw-rw-rw-  1 nobody  nobody    1, 8 Feb 24 15:43 random
drwxr-xr-x  2 johndoe johndoe     40 Feb 25 18:00 shm
lrwxrwxrwx  1 johndoe johndoe     15 Feb 25 18:00 stderr -> /proc/self/fd/2
lrwxrwxrwx  1 johndoe johndoe     15 Feb 25 18:00 stdin -> /proc/self/fd/0
lrwxrwxrwx  1 johndoe johndoe     15 Feb 25 18:00 stdout -> /proc/self/fd/1
crw-rw-rw-  1 nobody  nobody    5, 0 Feb 25 17:31 tty
crw-rw-rw-  1 nobody  nobody    1, 9 Feb 24 15:43 urandom
crw-rw-rw-  1 nobody  nobody    1, 5 Feb 24 15:43 zero
```

> [!NOTE]
> This is not a real linux filesystem type. It just bind mount a minimal set of device
> files in `CONTAINER_PATH`, such as `/dev/null`.


## --tmpfs

Mount new tmpfs on CONTAINER_PATH

```console,ignore
$ hakoniwa run --tmpfs /mytmp -- findmnt /mytmp
TARGET SOURCE FSTYPE OPTIONS
/mytmp tmpfs  tmpfs  rw,nosuid,nodev,relatime,uid=1000,gid=1000,inode64

$ hakoniwa run --tmpfs /mytmp -- touch /mytmp/myfile.txt
```
