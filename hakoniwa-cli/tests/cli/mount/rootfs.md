# --rootfs

Bind mount all subdirectories in ROOTFS to the container root with read-only access [default: /]

## host rootfs

```console
$ hakoniwa run -- ls /
bin
etc
lib
lib32
lib64
proc
sbin
usr

```

## customized rootfs

```console
$ hakoniwa run --rootfs ./tests/fixtures/rootfs -- /bin/ls /bin
arch
ash
base64
bbconfig
busybox
cat
chattr
chgrp
chmod
chown
cp
date
dd
df
dmesg
dnsdomainname
dumpkmap
echo
egrep
false
fatattr
fdflush
fgrep
fsync
getopt
grep
gunzip
gzip
hostname
ionice
iostat
ipcalc
kbd_mode
kill
link
linux32
linux64
ln
login
ls
lsattr
lzop
makemime
mkdir
mknod
mktemp
more
mount
mountpoint
mpstat
mv
netstat
nice
pidof
ping
ping6
pipe_progress
printenv
ps
pwd
reformime
rev
rm
rmdir
run-parts
sed
setpriv
setserial
sh
sleep
stat
stty
su
sync
tar
touch
true
umount
uname
usleep
watch
zcat

```

## none

```console
$ hakoniwa run --rootfs=none -b /bin -b /lib -b /lib64 -b /usr -- ls /
bin
lib
lib64
proc
usr

```
