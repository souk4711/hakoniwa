# --network

Configure network for the container (implies --unshare-network)

## pasta

```console
$ hakoniwa run --network pasta -- ip link
1: lo: <LOOPBACK,UP,LOWER_UP> mtu 65536 qdisc noqueue state UNKNOWN mode DEFAULT group default qlen 1000
    link/loopback 00:00:00:00:00:00 brd 00:00:00:00:00:00
2: [..]
[..]

```

```console
$ hakoniwa run --network pasta -- wget https://example.com --spider
...
HTTP request sent, awaiting response... 200 OK
Length: unspecified [text/html]
...
```
