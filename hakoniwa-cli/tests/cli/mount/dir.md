# --dir

Create a new dir on CONTAINER_PATH with 700 permissions (repeatable)

## can create dir

```console
$ hakoniwa run --dir /mytmp -- stat --printf %A /mytmp
drwx------
```
