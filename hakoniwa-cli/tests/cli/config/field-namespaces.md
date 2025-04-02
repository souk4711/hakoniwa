# CfgNamespace

## CfgNamespace#type `uts`

```console
$ hakoniwa run --config ./tests/fixtures/config/cfg-default.toml -- hostname
hakoniwa

```

## CfgNamespace#share `false`

```console
$ hakoniwa run --config ./tests/fixtures/config/cfg-default.toml -- ip link
1: lo: <LOOPBACK>[..]
...
```

## CfgNamespace#share `true`

```console
$ hakoniwa run --config ./tests/fixtures/config/field-namespaces.toml -- ip link
1: lo: <LOOPBACK,UP[..]
...
```

```console
$ hakoniwa run --config ./tests/fixtures/config/field-namespaces.toml -- aria2c https://example.com --async-dns-server=8.8.8.8 --dry-run
...
(OK):download completed.

```
