# Usage - COMMAND


## sh

```console
$ hakoniwa run
bash: cannot set terminal process group (-1): Inappropriate ioctl for device
bash: no job control in this shell
bash-5.1$ pwd
/
bash-5.1$ ls
bin  dev  lib  lib64  proc  usr
bash-5.1$ ls /dev
null  random  urandom  zero
bash-5.1$ ls /proc
1           bus        crypto         execdomains  ioports    kmsg           locks    mtrr          scsi      sys            uptime
2           cgroups    devices        fb           irq        kpagecgroup    meminfo  net           self      sysrq-trigger  version
acpi        cmdline    diskstats      filesystems  kallsyms   kpagecount     misc     pagetypeinfo  slabinfo  sysvipc        vmallocinfo
asound      config.gz  dma            fs           kcore      kpageflags     modules  partitions    softirqs  thread-self    vmstat
bootconfig  consoles   driver         interrupts   key-users  latency_stats  mounts   pressure      stat      timer_list     zoneinfo
buddyinfo   cpuinfo    dynamic_debug  iomem        keys       loadavg        mtd      schedstat     swaps     tty
bash-5.1$ exit
$
```


## ls

```console
$ hakoniwa run -- ls
bin  dev  lib  lib64  proc  usr

$ hakoniwa run --ro-bind /etc -- ls
bin  dev  etc  lib  lib64  proc  usr

$ hakoniwa run --ro-bind /etc:/myetc -- ls
bin  dev  myetc  lib  lib64  proc  usr

$ hakoniwa run --work-dir . -- ls
Cargo.lock  Cargo.toml  LICENSE-APACHE  LICENSE-MIT  README.md  hakoniwa  hakoniwa-cli

$ hakoniwa run --work-dir . -- ls /
bin  dev  hako  lib  lib64  proc  usr
```


## pwd

```console
$ hakoniwa run -- pwd
/

$ hakoniwa run --work-dir . -- pwd
/hako
```


## ps

```console
$ hakoniwa run -- ps aux
USER         PID %CPU %MEM    VSZ   RSS TTY      STAT START   TIME COMMAND
5001           1  0.0  0.0   6904  2104 ?        R+   05:32   0:00 ps aux
```


## whoami

```console
$ hakoniwa run -- whoami
whoami: cannot find name for user ID 5001: No such file or directory

$ hakoniwa run --ro-bind /etc/passwd -- whoami
johndoe

$ hakoniwa run --ro-bind /etc/passwd --uid 0 -- whoami
root
```


## hostname

```console
$ hakoniwa run -- hostname
hakoniwa

$ hakoniwa run --hostname myhostname -- hostname
myhostname
```


## ping

```console
$ hakoniwa run --share-net --ro-bind /etc/resolv.conf -- ping example.com -c 3
PING example.com(2606:2800:220:1:248:1893:25c8:1946 (2606:2800:220:1:248:1893:25c8:1946)) 56 data bytes
64 bytes from 2606:2800:220:1:248:1893:25c8:1946 (2606:2800:220:1:248:1893:25c8:1946): icmp_seq=1 ttl=54 time=252 ms
64 bytes from 2606:2800:220:1:248:1893:25c8:1946 (2606:2800:220:1:248:1893:25c8:1946): icmp_seq=2 ttl=54 time=177 ms
64 bytes from 2606:2800:220:1:248:1893:25c8:1946 (2606:2800:220:1:248:1893:25c8:1946): icmp_seq=3 ttl=54 time=298 ms

--- example.com ping statistics ---
3 packets transmitted, 3 received, 0% packet loss, time 2002ms
rtt min/avg/max/mdev = 176.668/242.301/297.814/49.972 ms
```


## wget

```console
$ hakoniwa run --share-net --ro-bind /etc/resolv.conf --work-dir . -- wget example.com
ERROR: could not open HSTS store. HSTS will be disabled.
--2022-08-12 06:25:25--  http://example.com/
Resolving example.com (example.com)... 2606:2800:220:1:248:1893:25c8:1946, 93.184.216.34
Connecting to example.com (example.com)|2606:2800:220:1:248:1893:25c8:1946|:80... connected.
HTTP request sent, awaiting response... 200 OK
Length: 1256 (1.2K) [text/html]
Saving to: 'index.html'

index.html                              100%[============================================================================>]   1.23K  --.-KB/s    in 0s

2022-08-12 06:25:27 (63.6 MB/s) - 'index.html' saved [1256/1256]
```


## env

```console
$ hakoniwa run -- env
TERM=xterm-256color

$ hakoniwa run --setenv EDITOR=vim -- env
TERM=xterm-256color
EDITOR=vim
```
