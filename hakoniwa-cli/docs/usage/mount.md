# Usage - Mount FileSystem

## --rootdir

Use ROOTDIR as the mount point for the container root fs

> [!NOTE]
> This method is mainly useful if you set it to a directory that contains a file system hierarchy, and want chroot into it.

> [!WARNING]
> Some empty directories/files that were used as mount point targets may be left behind even when the last process exits.

```console,ignore
$ mkdir -p rootfs && docker export $(docker create alpine) | tar -C rootfs -xf - && rmdir rootfs/proc

$ # --rootdir with RO options
$ hakoniwa run --rootfs=none --rootdir ./rootfs
/ $ cat /proc/1/mountinfo
438 250 254:0 /home/johndoe/rootfs / ro,relatime - ext4 /dev/mapper/cryptroot rw
251 438 0:61 / /proc rw,nosuid,nodev,noexec,relatime - proc proc rw
/ $ touch myfile.txt
touch: myfile.txt: Read-only file system

$ # --rootdir with RW options
$ hakoniwa run --rootfs=none --rootdir ./rootfs:rw
/ $ cat /proc/1/mountinfo
438 250 254:0 /home/johndoe/rootfs / rw,relatime - ext4 /dev/mapper/cryptroot rw
251 438 0:61 / /proc rw,nosuid,nodev,noexec,relatime - proc proc rw
/ $ touch myfile.txt
```

## --rootfs

Bind mount all subdirectories in ROOTFS to the container root with **read-only** access [default: **/**]

> [!NOTE]
> When use `/` as rootfs, it only mount following subdirectories: `/bin`, `/etc`, `/lib`, `/lib64`, `/lib32`, `/sbin`, `/usr`.

```console,ignore
$ mkdir -p rootfs && docker export $(docker create alpine) | tar -C rootfs -xf - && rmdir rootfs/proc

$ hakoniwa run --rootfs ./rootfs
/ $ cat /proc/1/mountinfo
438 250 0:33 /hakoniwa-jNBkqK / ro,nosuid,nodev - tmpfs tmpfs rw,size=8028428k,nr_inodes=1048576,inode64
470 438 254:0 /home/johndoe/rootfs/bin /bin ro,nosuid,relatime - ext4 /dev/mapper/cryptroot rw
471 438 254:0 /home/johndoe/rootfs/dev /dev ro,nosuid,relatime - ext4 /dev/mapper/cryptroot rw
472 438 254:0 /home/johndoe/rootfs/etc /etc ro,nosuid,relatime - ext4 /dev/mapper/cryptroot rw
473 438 254:0 /home/johndoe/rootfs/home /home ro,nosuid,relatime - ext4 /dev/mapper/cryptroot rw
474 438 254:0 /home/johndoe/rootfs/lib /lib ro,nosuid,relatime - ext4 /dev/mapper/cryptroot rw
475 438 254:0 /home/johndoe/rootfs/media /media ro,nosuid,relatime - ext4 /dev/mapper/cryptroot rw
476 438 254:0 /home/johndoe/rootfs/mnt /mnt ro,nosuid,relatime - ext4 /dev/mapper/cryptroot rw
477 438 254:0 /home/johndoe/rootfs/opt /opt ro,nosuid,relatime - ext4 /dev/mapper/cryptroot rw
481 438 254:0 /home/johndoe/rootfs/root /root ro,nosuid,relatime - ext4 /dev/mapper/cryptroot rw
482 438 254:0 /home/johndoe/rootfs/run /run ro,nosuid,relatime - ext4 /dev/mapper/cryptroot rw
483 438 254:0 /home/johndoe/rootfs/sbin /sbin ro,nosuid,relatime - ext4 /dev/mapper/cryptroot rw
484 438 254:0 /home/johndoe/rootfs/srv /srv ro,nosuid,relatime - ext4 /dev/mapper/cryptroot rw
485 438 254:0 /home/johndoe/rootfs/sys /sys ro,nosuid,relatime - ext4 /dev/mapper/cryptroot rw
486 438 254:0 /home/johndoe/rootfs/tmp /tmp ro,nosuid,relatime - ext4 /dev/mapper/cryptroot rw
487 438 254:0 /home/johndoe/rootfs/usr /usr ro,nosuid,relatime - ext4 /dev/mapper/cryptroot rw
488 438 254:0 /home/johndoe/rootfs/var /var ro,nosuid,relatime - ext4 /dev/mapper/cryptroot rw
251 438 0:61 / /proc rw,nosuid,nodev,noexec,relatime - proc proc rw
```

## --bindmount-ro (alias -b)

Bind mount the HOST_PATH on CONTAINER_PATH with **read-only** access (repeatable)

```console,ignore
$ hakoniwa run --bindmount-ro .:/mytmp -- findmnt /mytmp
TARGET SOURCE                                           FSTYPE OPTIONS
/mytmp /dev/mapper/cryptroot[/home/johndoe/MyContainer] ext4   ro,relatime

$ hakoniwa run --bindmount-ro .:/mytmp -- touch /mytmp/myfile.txt
touch: cannot touch '/mytmp/myfile.txt': Read-only file system

```

## --bindmount-rw (alias -B)

Bind mount the HOST_PATH on CONTAINER_PATH with **read-write** access (repeatable)

```console,ignore
$ hakoniwa run --bindmount-rw .:/mytmp -- findmnt /mytmp
TARGET SOURCE                                           FSTYPE OPTIONS
/mytmp /dev/mapper/cryptroot[/home/johndoe/MyContainer] ext4   rw,relatime

$ hakoniwa run --bindmount-rw .:/mytmp -- touch /mytmp/myfile.txt
$ file myfile.txt
myfile.txt: empty
```

## --devfs

Mount new devfs on CONTAINER_PATH (repeatable)

> [!NOTE]
> This is not a real linux filesystem type. It just bind mount a minimal set of device
> files in `CONTAINER_PATH`, such as `/dev/null`.

```console,ignore
$ hakoniwa run --devfs /mydev -- ls -lah /mydev
total 0
drwxr-xr-x 4 johndoe johndoe    340 Apr  8 18:15 .
drwxr-xr-x 6 johndoe johndoe    220 Apr  8 18:15 ..
crw------- 1 johndoe nobody  136, 2 Apr  8 18:15 console
lrwxrwxrwx 1 johndoe johndoe     11 Apr  8 18:15 core -> /proc/kcore
lrwxrwxrwx 1 johndoe johndoe     13 Apr  8 18:15 fd -> /proc/self/fd
crw-rw-rw- 1 nobody  nobody    1, 7 Apr  6 03:26 full
crw-rw-rw- 1 nobody  nobody    1, 3 Apr  6 03:26 null
lrwxrwxrwx 1 johndoe johndoe      8 Apr  8 18:15 ptmx -> pts/ptmx
drwxr-xr-x 2 nobody  nobody       0 Apr  8 18:15 pts
crw-rw-rw- 1 nobody  nobody    1, 8 Apr  6 03:26 random
drwxr-xr-x 2 johndoe johndoe     40 Apr  8 18:15 shm
lrwxrwxrwx 1 johndoe johndoe     15 Apr  8 18:15 stderr -> /proc/self/fd/2
lrwxrwxrwx 1 johndoe johndoe     15 Apr  8 18:15 stdin -> /proc/self/fd/0
lrwxrwxrwx 1 johndoe johndoe     15 Apr  8 18:15 stdout -> /proc/self/fd/1
crw-rw-rw- 1 nobody  nobody    5, 0 Apr  8 17:07 tty
crw-rw-rw- 1 nobody  nobody    1, 9 Apr  6 03:26 urandom
crw-rw-rw- 1 nobody  nobody    1, 5 Apr  6 03:26 zero
```

## --tmpfs

Mount new tmpfs on CONTAINER_PATH (repeatable)

```console,ignore
$ hakoniwa run --tmpfs /mytmp -- findmnt /mytmp
TARGET SOURCE FSTYPE OPTIONS
/mytmp tmpfs  tmpfs  rw,nosuid,nodev,relatime,uid=1000,gid=1000,inode64

$ hakoniwa run --tmpfs /mytmp -- touch /mytmp/myfile.txt
```

## --dir

Create a new dir on CONTAINER_PATH with 700 permissions (repeatable)

## --symlink

Create a symbolic link on LINK_PATH pointing to the ORIGINAL_PATH (repeatable)

```console,ignore
$ hakoniwa run --symlink opt/dart-sdk/bin:/mybin -- ls -lah /mybin
lrwxrwxrwx 1 johndoe johndoe 16 Apr  8 18:23 /mybin -> opt/dart-sdk/bin
```
