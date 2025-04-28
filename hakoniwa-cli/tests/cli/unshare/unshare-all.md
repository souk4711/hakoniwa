# --unshare-all

Create new CGROUP, IPC, NETWORK, UTS, ... namespaces

## new NETWORK namespace

```console
$ hakoniwa run --unshare-all -- ip link
1: lo: <LOOPBACK> mtu 65536 qdisc noop state DOWN mode DEFAULT group default qlen 1000
    link/loopback 00:00:00:00:00:00 brd 00:00:00:00:00:00

```
