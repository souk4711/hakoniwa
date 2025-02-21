# --limit-fsize

Limit the maximum size in bytes of files that the COMMAND may create

## file too large

```console
$ hakoniwa run --bindmount /dev:/dev --limit-fsize 2 -- dd if=/dev/random of=output.txt count=1 bs=4
? 1
[..]: error writing 'output.txt': File too large
1+0 records in
0+0 records out
2 bytes copied, [..]

```
