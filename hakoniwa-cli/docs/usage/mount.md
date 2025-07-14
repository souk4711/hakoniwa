# Usage - Mount FileSystem

## --rootdir

Use ROOTDIR as the mount point for the container root fs

> [!NOTE]
> This method is mainly useful if you set it to a directory that contains a file system hierarchy, and want chroot into it.

> [!WARNING]
> Some empty directories/files that were used as mount point targets may be left behind even when the last process exits.

```console,ignore
$ mkdir -p rootfs && podman export $(podman create archlinux) | tar -C rootfs -xf -

$ hakoniwa run --rootdir ./rootfs    -- findmnt
TARGET  SOURCE                                      FSTYPE OPTIONS
/       /dev/mapper/cryptroot[/home/johndoe/rootfs] ext4   ro,relatime
`-/proc proc                                        proc   rw,nosuid,nodev,noexec,relatime

$ hakoniwa run --rootdir ./rootfs:rw -- findmnt
TARGET  SOURCE                                      FSTYPE OPTIONS
/       /dev/mapper/cryptroot[/home/johndoe/rootfs] ext4   rw,relatime
`-/proc proc                                        proc   rw,nosuid,nodev,noexec,relatime
```

## --rootfs

Bind mount all subdirectories in ROOTFS to the container root with **read-only** access [default: **/**]

> [!NOTE]
> When use `/` as rootfs, it only mount following subdirectories: `/bin`, `/etc`, `/lib`, `/lib64`, `/lib32`, `/sbin`, `/usr`.

```console,ignore
$ # Run with /
$ hakoniwa run --rootfs / -- ls -lah
total 16K
drwxr-xr-x   5 johndoe johndoe  200 Jul 14 02:08 .
drwxr-xr-x   5 johndoe johndoe  200 Jul 14 02:08 ..
lrwxrwxrwx   1 johndoe johndoe    7 Jul 14 02:08 bin -> usr/bin
drwxr-xr-x 141 nobody  nobody   12K Jul 13 04:53 etc
lrwxrwxrwx   1 johndoe johndoe    7 Jul 14 02:08 lib -> usr/lib
lrwxrwxrwx   1 johndoe johndoe    9 Jul 14 02:08 lib32 -> usr/lib32
lrwxrwxrwx   1 johndoe johndoe    7 Jul 14 02:08 lib64 -> usr/lib
dr-xr-xr-x 352 nobody  nobody     0 Jul 14 02:08 proc
lrwxrwxrwx   1 johndoe johndoe    7 Jul 14 02:08 sbin -> usr/bin
drwxr-xr-x  10 nobody  nobody  4.0K Jul 10 15:38 usr

$ # Run with customized rootfs
$ hakoniwa run --rootfs ./rootfs -- ls -lah
total 52K
drwxr-xr-x  16   1000   1000  400 Jul 14 16:02 .
drwxr-xr-x  16   1000   1000  400 Jul 14 16:02 ..
lrwxrwxrwx   1   1000   1000    7 Jul 14 16:02 bin -> usr/bin
drwxr-xr-x   2   1000   1000 4.0K May  3 19:26 boot
drwxr-xr-x   2   1000   1000 4.0K Jul  7 20:41 dev
drwxr-xr-x  38   1000   1000 4.0K Jul  7 20:41 etc
drwxr-xr-x   2   1000   1000 4.0K May  3 19:26 home
lrwxrwxrwx   1   1000   1000    7 Jul 14 16:02 lib -> usr/lib
lrwxrwxrwx   1   1000   1000    7 Jul 14 16:02 lib64 -> usr/lib
drwxr-xr-x   2   1000   1000 4.0K May  3 19:26 mnt
drwxr-xr-x   2   1000   1000 4.0K May  3 19:26 opt
dr-xr-xr-x 353 nobody nobody    0 Jul 14 16:02 proc
drwxr-x---   2   1000   1000 4.0K May  3 19:26 root
drwxr-xr-x   2   1000   1000 4.0K May  3 19:26 run
lrwxrwxrwx   1   1000   1000    7 Jul 14 16:02 sbin -> usr/bin
drwxr-xr-x   4   1000   1000 4.0K Jul  6 00:03 srv
drwxr-xr-x   2   1000   1000 4.0K Jul  7 20:41 sys
drwxr-xr-x   2   1000   1000 4.0K May  3 19:26 tmp
drwxr-xr-x   8   1000   1000 4.0K Jul  6 00:03 usr
drwxr-xr-x  12   1000   1000 4.0K Jul  6 00:03 var

$ # Run with `none`
$ hakoniwa run --rootfs=none -b /bin -b /lib -b /lib64 -b /usr -- ls -lah
total 880K
drwxr-xr-x   7  1000  1000  140 Jul 13 18:03 .
drwxr-xr-x   7  1000  1000  140 Jul 13 18:03 ..
drwxr-xr-x   7 65534 65534 160K Jul 13 16:45 bin
drwxr-xr-x 249 65534 65534 352K Jul 10 07:38 lib
drwxr-xr-x 249 65534 65534 352K Jul 10 07:38 lib64
dr-xr-xr-x 354 65534 65534    0 Jul 13 18:03 proc
drwxr-xr-x  10 65534 65534 4.0K Jul 10 07:38 usr
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
