# CfgLandlock

## CfgLandlockFsRule#perm `r--`

```console
$ hakoniwa run --config ./tests/fixtures/config/field-landlock.toml -- ls /tmp-r

$ hakoniwa run --config ./tests/fixtures/config/field-landlock.toml -- touch /tmp-r/myfile.txt
? 1
[..] Permission denied

$ hakoniwa run --config ./tests/fixtures/config/field-landlock.toml -- sh -c "cp /bin/echo /tmp-r/echo && /tmp-r/echo"
? 1
[..] Permission denied

```

## CfgLandlockFsRule#perm `rw-`

```console
$ hakoniwa run --config ./tests/fixtures/config/field-landlock.toml -- ls /tmp-rw

$ hakoniwa run --config ./tests/fixtures/config/field-landlock.toml -- touch /tmp-rw/myfile.txt

$ hakoniwa run --config ./tests/fixtures/config/field-landlock.toml -- sh -c "cp /bin/echo /tmp-rw/echo && /tmp-rw/echo"
? 126
[..] Permission denied

```

## CfgLandlockFsRule#perm `rwx`

```console
$ hakoniwa run --config ./tests/fixtures/config/field-landlock.toml -- ls /tmp-rwx

$ hakoniwa run --config ./tests/fixtures/config/field-landlock.toml -- touch /tmp-rwx/myfile.txt

$ hakoniwa run --config ./tests/fixtures/config/field-landlock.toml -- sh -c "cp /bin/echo /tmp-rwx/echo && /tmp-rwx/echo"


```

## CfgLandlockFsRule#perm `-w-`

```console
$ hakoniwa run --config ./tests/fixtures/config/field-landlock.toml -- ls /tmp-w
? 2
[..] Permission denied

$ hakoniwa run --config ./tests/fixtures/config/field-landlock.toml -- touch /tmp-w/myfile.txt

$ hakoniwa run --config ./tests/fixtures/config/field-landlock.toml -- sh -c "cp /bin/echo /tmp-w/echo && /tmp-w/echo"
? 126
[..] Permission denied

```

## CfgLandlockFsRule#perm `-wx`

```console
$ hakoniwa run --config ./tests/fixtures/config/field-landlock.toml -- ls /tmp-wx
? 2
[..] Permission denied

$ hakoniwa run --config ./tests/fixtures/config/field-landlock.toml -- touch /tmp-wx/myfile.txt

$ hakoniwa run --config ./tests/fixtures/config/field-landlock.toml -- sh -c "cp /bin/echo /tmp-wx/echo && /tmp-wx/echo"
? 126
[..] Permission denied

```
