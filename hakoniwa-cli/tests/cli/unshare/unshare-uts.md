# --unshare-uts

Create new UTS namespace

## sethostname

```console
$ hakoniwa run --unshare-uts --uidmap 0 -- sh -c "hostname myhost && hostname"
myhost

```
