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

## --bindmount

Bind mount the HOST_PATH on CONTAINER_PATH with read-write access

```console
$ hakoniwa run --bindmount $PWD:/mytmp -- touch /mytmp/myfile.txt
$ file myfile.txt
myfile.txt: empty

$ hakoniwa run --bindmount $PWD:/mytmp -- findmnt /mytmp
TARGET SOURCE                                           FSTYPE OPTIONS
/mytmp /dev/mapper/cryptroot[/home/johndoe/MyContainer] ext4   rw,relatime

```

## --bindmount-ro

Bind mount the HOST_PATH on CONTAINER_PATH with read-only access

```console
$ hakoniwa run --bindmount-ro $PWD:/mytmp -- touch /mytmp/myfile.txt
touch: cannot touch '/mytmp/myfile.txt': Read-only file system

$ hakoniwa run --bindmount-ro $PWD:/mytmp -- findmnt /mytmp
TARGET SOURCE                                           FSTYPE OPTIONS
/mytmp /dev/mapper/cryptroot[/home/johndoe/MyContainer] ext4   ro,relatime
```

## --tmpfsmount

Mount new tmpfs on CONTAINER_PATH
