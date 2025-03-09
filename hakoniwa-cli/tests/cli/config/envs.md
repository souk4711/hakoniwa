# CfgEnv

## CfgEnv#value

```console
$ hakoniwa run --config ./tests/config/envs.toml -- env
...
DISPLAY=:1
...

```

## CfgEnv#value NULL

```console
$ ENV_INHERIT=123456 hakoniwa run --config ./tests/config/envs.toml -- env
...
ENV_INHERIT=123456
...

```

## CfgEnv#value BLANK str

```console
$ ENV_CLEARED=123456 hakoniwa run --config ./tests/config/envs.toml -- env
...
ENV_CLEARED=
...

```
