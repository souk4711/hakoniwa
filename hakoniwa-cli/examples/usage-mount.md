# Usage - Mount FileSystem

## --rootfs

Bind mount all necessary subdirectories in ROOTFS to the container root with read-only access [default: /]

```console
$ mkdir -p rootfs && docker export $(docker create alpine) | tar -C rootfs -xf -
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

## --bindmount

Bind mount the HOST_PATH on CONTAINER_PATH with read-write access

```console
$ hakoniwa run --bindmount $PWD:/mytmp -- findmnt /mytmp
TARGET SOURCE                                           FSTYPE OPTIONS
/mytmp /dev/mapper/cryptroot[/home/johndoe/MyContainer] ext4   rw,relatime

$ hakoniwa run --bindmount $PWD:/mytmp -- touch /mytmp/myfile.txt
$ file myfile.txt
myfile.txt: empty
```

## --bindmount-ro

Bind mount the HOST_PATH on CONTAINER_PATH with read-only access

```console
$ hakoniwa run --bindmount-ro $PWD:/mytmp -- findmnt /mytmp
TARGET SOURCE                                           FSTYPE OPTIONS
/mytmp /dev/mapper/cryptroot[/home/johndoe/MyContainer] ext4   ro,relatime

$ hakoniwa run --bindmount-ro $PWD:/mytmp -- touch /mytmp/myfile.txt
touch: cannot touch '/mytmp/myfile.txt': Read-only file system

```

## --tmpfsmount

Mount new tmpfs on CONTAINER_PATH

```console
$ hakoniwa run --tmpfsmount /mytmp -- findmnt /mytmp
TARGET SOURCE FSTYPE OPTIONS
/mytmp tmpfs  tmpfs  rw,nosuid,nodev,noexec,relatime,uid=1000,gid=1000,inode64

$ hakoniwa run --tmpfsmount /mytmp --uidmap 1000 --gidmap 1000 -- touch /mytmp/myfile.txt
$ echo $?
0
```
