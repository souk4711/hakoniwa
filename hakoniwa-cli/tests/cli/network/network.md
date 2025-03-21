# --network

Configure network for the container

## none

```console
$ hakoniwa run --network none -- ip link
1: lo: <LOOPBACK> mtu 65536 qdisc noop state DOWN mode DEFAULT group default qlen 1000
    link/loopback 00:00:00:00:00:00 brd 00:00:00:00:00:00

```

## host

```console
$ hakoniwa run --network host -- ip link
1: lo: <LOOPBACK,UP,LOWER_UP> mtu 65536 qdisc noqueue state UNKNOWN mode DEFAULT group default qlen 1000
    link/loopback 00:00:00:00:00:00 brd 00:00:00:00:00:00
2: [..]
...
3: [..]
...
```

```console
$ hakoniwa run --network host -- curl -s --head https://example.com
[..]200[..]
content-type: text/html
...
```

## pasta

```console
$ hakoniwa run --network pasta -- ip link
1: lo: <LOOPBACK,UP,LOWER_UP> mtu 65536 qdisc noqueue state UNKNOWN mode DEFAULT group default qlen 1000
    link/loopback 00:00:00:00:00:00 brd 00:00:00:00:00:00
2: [..]
[..]

```

```console
$ hakoniwa run --network pasta -- curl -s --head https://example.com
[..]200[..]
content-type: text/html
...
```
