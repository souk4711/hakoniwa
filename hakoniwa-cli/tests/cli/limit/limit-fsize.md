# --limit-fsize

Limit the maximum size in bytes of files that the COMMAND may create

## file too large

```console
$ hakoniwa run --devfs /dev --tmpfs /tmp --limit-fsize 2 -- dd if=/dev/random of=/tmp/output.txt count=1 bs=4
? 1
[..]: error writing '/tmp/output.txt': File too large
1+0 records in
0+0 records out
2 bytes copied, [..]

```
