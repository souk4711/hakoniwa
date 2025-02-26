# Usage - Unshare Linux Namespace

## --unshare-cgroup

Create new CGROUP namespace

## --unshare-ipc

Create new IPC namespace

## --unshare-network

Create new NETWORK namespace

```console
$ hakoniwa run --unshare-network -- ip link
1: lo: <LOOPBACK> mtu 65536 qdisc noop state DOWN mode DEFAULT group default qlen 1000
    link/loopback 00:00:00:00:00:00 brd 00:00:00:00:00:00

```

## --unshare-uts

Create new UTS namespace

```console
$ hakoniwa run --unshare-uts -- hostname
hakoniwa

```
