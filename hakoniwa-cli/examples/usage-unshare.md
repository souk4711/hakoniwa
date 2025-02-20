# Usage - Linux Namespace

## --unshare-network

Create new NETWORK namespace

```console,ignore
$ hakoniwa run --unshare-network -- ip link
1: lo: <LOOPBACK> mtu 65536 qdisc noop state DOWN mode DEFAULT group default qlen 1000
    link/loopback 00:00:00:00:00:00 brd 00:00:00:00:00:00
```

## --unshare-uts

Create new UTS namespace

```console,ignore
$ hakoniwa run --unshare-uts --uidmap 0 --gidmap 0 -- sh
sh: cannot set terminal process group (-1): Inappropriate ioctl for device
sh: no job control in this shell
sh-5.2# hostname myhost
sh-5.2# hostname
myhost
sh-5.2#
```
