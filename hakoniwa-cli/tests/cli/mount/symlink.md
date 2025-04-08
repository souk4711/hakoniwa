# --symlink

Create a symbolic link on LINK_PATH pointing to the ORIGINAL_PATH (repeatable)

## can create link

```console
$ hakoniwa run --tmpfs /tmp --symlink /tmp:/mytmp -- touch /mytmp/myfile.txt

```
