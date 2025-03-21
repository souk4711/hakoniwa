# Usage - Network

Configure network for the container (implies --unshare-network)

## --network

### pasta

```console,ignore
$ hakoniwa run --network pasta -- ip link
1: lo: <LOOPBACK,UP,LOWER_UP> mtu 65536 qdisc noqueue state UNKNOWN mode DEFAULT group default qlen 1000
    link/loopback 00:00:00:00:00:00 brd 00:00:00:00:00:00
2: wlan0: <BROADCAST,MULTICAST,UP,LOWER_UP> mtu 65520 qdisc fq_codel state UNKNOWN mode DEFAULT group default qlen 1000
    link/ether da:2a:89:14:17:45 brd ff:ff:ff:ff:ff:ff

$ # Access network
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

$ # Forward host port 8080 to 8080 in the container
$ hakoniwa run --network pasta:-t,8080 -- darkhttpd .
darkhttpd/1.16, copyright (c) 2003-2024 Emil Mikulic.
listening on: http://0.0.0.0:8080/
192.168.2.82 - - [21/Mar/2025:15:52:57 +0800] "GET / HTTP/1.1" 200 615 "" "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/134.0.0.0 Safari/537.36 Edg/134.0.0.0"
```
