# CfgNamespace

## CfgNamespace#type `uts`

```console
$ hakoniwa run --config ./tests/config/cfg-default.toml -- hostname
hakoniwa

```

## CfgNamespace#share `false`

```console
$ hakoniwa run --config ./tests/config/cfg-default.toml -- ip link
1: lo: <LOOPBACK>[..]
...
```

## CfgNamespace#share `true`

```console
$ hakoniwa run --config ./tests/config/field-namespaces.toml -- ip link
1: lo: <LOOPBACK,UP[..]
...
```
