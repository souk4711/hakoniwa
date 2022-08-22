# Usage - Linux Namespace


## --share-net

Retain the NETWORK namespace

```sh
$ hakoniwa run -- ping 127.0.0.1 -c 3
$ echo $?
2

$ hakoniwa run --share-net -- ping 127.0.0.1 -c 3
PING 127.0.0.1 (127.0.0.1) 56(84) bytes of data.
64 bytes from 127.0.0.1: icmp_seq=1 ttl=64 time=0.080 ms
64 bytes from 127.0.0.1: icmp_seq=2 ttl=64 time=0.076 ms
64 bytes from 127.0.0.1: icmp_seq=3 ttl=64 time=0.068 ms

--- 127.0.0.1 ping statistics ---
3 packets transmitted, 3 received, 0% packet loss, time 2038ms
rtt min/avg/max/mdev = 0.068/0.074/0.080/0.005 ms
```


## --share-uts

Retain the UTS namespace

```sh
$ hakoniwa run -- hostname
hakoniwa

$ hakoniwa run --share-uts -- hostname
archlinux

$ hostname
archlinux
```
