# CfgLandlock

## CfgLandlockResource#unrestrict `false`

```console
$ hakoniwa run --config ./tests/fixtures/config/field-landlock.toml -- /bin/python3 -m http.server
? 1
...
[..] Permission denied
...

```

## CfgLandlockResource#unrestrict `true`

```console
$ hakoniwa run --config ./tests/fixtures/config/field-landlock.toml -- aria2c https://example.com --async-dns-server=8.8.8.8 --dry-run
...
(OK):download completed.

```

## CfgLandlockFsRule#access `r--`

```console
$ hakoniwa run --config ./tests/fixtures/config/field-landlock.toml -- ls /tmp-r

```

```console
$ hakoniwa run --config ./tests/fixtures/config/field-landlock.toml -- touch /tmp-r/myfile.txt
? 1
[..] Permission denied

```

```console
$ hakoniwa run --config ./tests/fixtures/config/field-landlock.toml -- sh -c "cp /bin/echo /tmp-r/echo && /tmp-r/echo"
? 1
[..] Permission denied

```

## CfgLandlockFsRule#access `rw-`

```console
$ hakoniwa run --config ./tests/fixtures/config/field-landlock.toml -- ls /tmp-rw

```

```console
$ hakoniwa run --config ./tests/fixtures/config/field-landlock.toml -- touch /tmp-rw/myfile.txt

```

```console
$ hakoniwa run --config ./tests/fixtures/config/field-landlock.toml -- sh -c "cp /bin/echo /tmp-rw/echo && /tmp-rw/echo"
? 126
[..] Permission denied

```

## CfgLandlockFsRule#access `rwx`

```console
$ hakoniwa run --config ./tests/fixtures/config/field-landlock.toml -- ls /tmp-rwx

```

```console
$ hakoniwa run --config ./tests/fixtures/config/field-landlock.toml -- touch /tmp-rwx/myfile.txt

```

```console
$ hakoniwa run --config ./tests/fixtures/config/field-landlock.toml -- sh -c "cp /bin/echo /tmp-rwx/echo && /tmp-rwx/echo"


```

## CfgLandlockFsRule#access `-w-`

```console
$ hakoniwa run --config ./tests/fixtures/config/field-landlock.toml -- ls /tmp-w
? 2
[..] Permission denied

```

```console
$ hakoniwa run --config ./tests/fixtures/config/field-landlock.toml -- touch /tmp-w/myfile.txt

```

```console
$ hakoniwa run --config ./tests/fixtures/config/field-landlock.toml -- sh -c "cp /bin/echo /tmp-w/echo && /tmp-w/echo"
? 126
[..] Permission denied

```

## CfgLandlockFsRule#access `-wx`

```console
$ hakoniwa run --config ./tests/fixtures/config/field-landlock.toml -- ls /tmp-wx
? 2
[..] Permission denied

```

```console
$ hakoniwa run --config ./tests/fixtures/config/field-landlock.toml -- touch /tmp-wx/myfile.txt

```

```console
$ hakoniwa run --config ./tests/fixtures/config/field-landlock.toml -- sh -c "cp /bin/echo /tmp-wx/echo && /tmp-wx/echo"
? 126
[..] Permission denied

```
