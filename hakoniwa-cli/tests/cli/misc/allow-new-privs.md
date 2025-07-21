# --allow-new-privs

Set the NoNewPrivileges flag to off

## unset NoNewPrivileges

```console
$ hakoniwa run --allow-new-privs -- cat /proc/self/status
...
NoNewPrivs:[..]0
Seccomp:[..]2
Seccomp_filters:[..]1
...
```
