# --limit-nofile

Limit the maximum file descriptor number that can be opened by the COMMAND

# cannot open shared object file

```console
$ hakoniwa run --limit-nofile 2 -- echo
? 127
[..] cannot open shared object file[..]
...

```
