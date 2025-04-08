# Usage - Landlock

## --landlock-restrict

### fs

Filesystem restrictions, this feature requires minimum kernel version 5.13.

```console,ignore
$ hakoniwa run --landlock-restrict fs --landlock-fs-rx /bin:/lib -- cat /etc/hosts
cat: /etc/hosts: Permission denied

```

### tcp.bind

Network TCP restrictions, this feature requires minimum kernel version 6.7.

```console,ignore
$ hakoniwa run --landlock-restrict tcp.bind -- python3 -m http.server
Traceback (most recent call last):
...
  File "/usr/lib/python3.13/socketserver.py", line 478, in server_bind
    self.socket.bind(self.server_address)
    ~~~~~~~~~~~~~~~~^^^^^^^^^^^^^^^^^^^^^
PermissionError: [Errno 13] Permission denied

```

### tcp.connect

Network TCP restrictions, this feature requires minimum kernel version 6.7.

```console,ignore
$ hakoniwa run --landlock-restrict tcp.connect -- aria2c https://example.com --dry-run

04/02 18:39:21 [NOTICE] Downloading 1 item(s)

04/02 18:39:21 [ERROR] CUID#7 - Download aborted. URI=https://example.com
Exception: [AbstractCommand.cc:351] errorCode=1 URI=https://example.com
  -> [SocketCore.cc:507] errorCode=1 Failed to connect to the host 23.215.0.138, cause: Permission denied

04/02 18:39:21 [NOTICE] Download GID#a148517f17eaf915 not complete:

Download Results:
gid   |stat|avg speed  |path/URI
======+====+===========+=======================================================
a14851|ERR |       0B/s|https://example.com

Status Legend:
(ERR):error occurred.

aria2 will resume download if the transfer is restarted.
If there are any errors, then see the log file. See '-l' option in help/man page for details.

```

## --landlock-fs-ro

Allow to read files beneath PATH (implies **--landlock-restrict=fs**)

```console,ignore
$ hakoniwa run --landlock-fs-rx /bin:/lib --landlock-fs-ro /etc -- cat /etc/hosts
# Static table lookup for hostnames.
# See hosts(5) for details.

```

## --landlock-fs-rw

Allow to read-write files beneath PATH (implies **--landlock-restrict=fs**)

```console
$ hakoniwa run --tmpfs /tmp --landlock-fs-rx /bin:/lib --landlock-fs-rw /tmp -- touch /tmp/myfile.txt

```

## --landlock-fs-rx

Allow to execute files beneath PATH (implies **--landlock-restrict=fs**)

## --landlock-tcp-bind

Allow binding a TCP socket to a local port (implies **--landlock-restrict=tcp.bind**)

```console,ignore
$ hakoniwa run --landlock-tcp-bind 8000 -- python3 -m http.server
Serving HTTP on 0.0.0.0 port 8000 (http://0.0.0.0:8000/) ...

```

## --landlock-tcp-connect

Allow connecting an active TCP socket to a remote port (implies **--landlock-restrict=tcp.connect**)

```console,ignore
$ hakoniwa run --landlock-tcp-connect 443 -- aria2c https://example.com --dry-run

04/01 18:45:25 [NOTICE] Downloading 1 item(s)
[#7e2eec 0B/0B CN:0]
04/01 18:45:26 [NOTICE] Download complete: /index.html

Download Results:
gid   |stat|avg speed  |path/URI
======+====+===========+=======================================================
7e2eec|OK  |       0B/s|/index.html

Status Legend:
(OK):download completed.

```
