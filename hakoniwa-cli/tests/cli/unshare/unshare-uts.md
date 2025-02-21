# --unshare-uts

Create new UTS namespace

## allow sethostname

```console
$ hakoniwa run --unshare-uts --uidmap 0 -- sh -c "hostname myhost && hostname"
myhost

```
