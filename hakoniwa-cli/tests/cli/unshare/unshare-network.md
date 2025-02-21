# --unshare-network

Create new NETWORK namespace

## no network interfaces except a new loopback interface

```console
$ hakoniwa run --unshare-network -- ip link
1: lo: <LOOPBACK> mtu 65536 qdisc noop state DOWN mode DEFAULT group default qlen 1000
    link/loopback 00:00:00:00:00:00 brd 00:00:00:00:00:00

```
