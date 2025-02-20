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

```console
$ hakoniwa run --unshare-uts --uidmap 0 --gidmap 0 -- sh -c "hostname myhost && hostname"
myhost

```
