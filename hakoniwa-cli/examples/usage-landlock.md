# Usage - Landlock

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
$ hakoniwa run --landlock-tcp-bind 8000 -- /bin/python3 -m http.server
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
