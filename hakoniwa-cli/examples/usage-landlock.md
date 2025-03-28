# Usage - Landlock

## --landlock-restrict-fs

Restrict access rights to the entire file system

```console,ignore
$ hakoniwa run --landlock-restrict-fs -- echo
hakoniwa: execve("/usr/bin/echo", ["/usr/bin/echo"], []) => EACCES: Permission denied

$ hakoniwa run --landlock-restrict-fs --landlock-fs-rx /bin:/lib -- echo


```

## --landlock-fs-ro

Allow to read files beneath PATH (implies **--landlock-restrict-fs**)

```console,ignore
$ hakoniwa run --landlock-fs-rx /bin:/lib --landlock-fs-ro /etc -- cat /etc/hosts
# Static table lookup for hostnames.
# See hosts(5) for details.

```

## --landlock-fs-rw

Allow to read-write files beneath PATH (implies **--landlock-restrict-fs**)

```console
$ hakoniwa run --tmpfs /tmp --landlock-fs-rx /bin:/lib --landlock-fs-rw /tmp -- touch /tmp/myfile.txt

```

## --landlock-fs-rx

Allow to execute files beneath PATH (implies **--landlock-restrict-fs**)
