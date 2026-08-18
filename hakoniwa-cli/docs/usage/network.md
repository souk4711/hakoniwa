# Usage - Network

## --network

Configure network for the container

### none

Create a network namespace for the container but do not configure network interfaces for
it, thus the container has no network connectivity.

> [!NOTE]
> This is equivalent to running with `--unshare-nework` option.

```console
$ hakoniwa run --network none -- ip link
1: lo: <LOOPBACK> mtu 65536 qdisc noop state DOWN mode DEFAULT group default qlen 1000
    link/loopback 00:00:00:00:00:00 brd 00:00:00:00:00:00

```

### host

Do not create a network namespace, the container uses the host’s network. Note: The host
mode gives the container full access to local system services such as D-bus and is
therefore considered insecure.

> [!NOTE]
> This is equivalent to running without `--unshare-nework` option.

```console,ignore
$ hakoniwa run --network host -- ip link
1: lo: <LOOPBACK,UP,LOWER_UP> mtu 65536 qdisc noqueue state UNKNOWN mode DEFAULT group default qlen 1000
    link/loopback 00:00:00:00:00:00 brd 00:00:00:00:00:00
2: enp86s0: <NO-CARRIER,BROADCAST,MULTICAST,UP> mtu 1500 qdisc fq_codel state DOWN mode DEFAULT group default qlen 1000
    link/ether cc:30:80:3c:79:83 brd ff:ff:ff:ff:ff:ff
    altname enxcc30803c7983
3: wlan0: <BROADCAST,MULTICAST,UP,LOWER_UP> mtu 1500 qdisc noqueue state UP mode DORMANT group default qlen 1000
    link/ether 14:85:7f:08:5b:2e brd ff:ff:ff:ff:ff:ff

```

### pasta

Use [pasta(1)](https://passt.top) to create a user-mode networking stack.

By default, IPv4 and IPv6 addresses and routes are copied from the host.
[OPTIONS](https://passt.top/builds/latest/web/passt.1.html) described in
pasta(1) can be specified as comma-separated arguments.

In terms of pasta(1) options, **--config-net** is given by default, in order to configure
networking when the container is started, and **--no-map-gw** is also assumed by default,
to avoid direct access from container to host using the gateway address. The latter can
be overridden by passing **--map-gw** in the pasta-specific options (despite not being
an actual pasta(1) option).

Also, **-t none** and **-u none** are passed if, respectively, no TCP or UDP port forwarding
from host to container is configured, to disable automatic port forwarding based on bound
ports. Similarly, **-T none** and **-U none** are given to disable the same functionality
from container to host.

```console,ignore
$ hakoniwa run --network pasta -- ip link
1: lo: <LOOPBACK,UP,LOWER_UP> mtu 65536 qdisc noqueue state UNKNOWN mode DEFAULT group default qlen 1000
    link/loopback 00:00:00:00:00:00 brd 00:00:00:00:00:00
2: wlan0: <BROADCAST,MULTICAST,UP,LOWER_UP> mtu 65520 qdisc fq_codel state UNKNOWN mode DEFAULT group default qlen 1000
    link/ether da:2a:89:14:17:45 brd ff:ff:ff:ff:ff:ff

$ hakoniwa run --network pasta -- wget https://example.com --spider
Spider mode enabled. Check if remote file exists.
--2025-03-21 15:51:02--  https://example.com/
Loaded CA certificate '/etc/ssl/certs/ca-certificates.crt'
Resolving example.com (example.com)... 2600:1406:3a00:21::173e:2e65, 2600:1408:ec00:36::1736:7f31, 2600:1406:bc00:53::b81e:94c8, ...
Connecting to example.com (example.com)|2600:1406:3a00:21::173e:2e65|:443... connected.
HTTP request sent, awaiting response... 200 OK
Length: unspecified [text/html]
Remote file exists and could contain further links,
but recursion is disabled -- not retrieving.

$ hakoniwa run --network pasta:-T,7890 -- wget https://example.com --spider -e use_proxy=on -e https_proxy=http://127.0.0.1:7890
Spider mode enabled. Check if remote file exists.
--2025-03-21 22:00:02--  https://example.com/
Loaded CA certificate '/etc/ssl/certs/ca-certificates.crt'
Connecting to 127.0.0.1:7890... connected.
Proxy request sent, awaiting response... 200 OK
Length: unspecified [text/html]
Remote file exists and could contain further links,
but recursion is disabled -- not retrieving.

$ hakoniwa run --network pasta:-t,8080 -- darkhttpd .
darkhttpd/1.16, copyright (c) 2003-2024 Emil Mikulic.
listening on: http://0.0.0.0:8080/
192.168.2.82 - - [21/Mar/2025:15:52:57 +0800] "GET / HTTP/1.1" 200 615 "" "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/134.0.0.0 Safari/537.36 Edg/134.0.0.0"
```
