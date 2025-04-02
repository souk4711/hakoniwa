# CfgNetwork

## CfgNetwork#mode `pasta`

```console
$ hakoniwa run --config ./tests/fixtures/config/field-network.toml -- ip link
1: lo: <LOOPBACK,UP,LOWER_UP> mtu 65536 qdisc noqueue state UNKNOWN mode DEFAULT group default qlen 1000
    link/loopback 00:00:00:00:00:00 brd 00:00:00:00:00:00
2: [..]
[..]

```

```console
$ hakoniwa run --config ./tests/fixtures/config/field-network.toml -- aria2c https://example.com --async-dns-server=8.8.8.8 --dry-run
...
(OK):download completed.

```
