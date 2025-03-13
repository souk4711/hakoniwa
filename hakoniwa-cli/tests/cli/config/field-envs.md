# CfgEnv

## CfgEnv#value

```console
$ hakoniwa run --config ./tests/config/field-envs.toml -- env
...
DISPLAY=:1
...

```

## CfgEnv#value `NULL`

```console
$ ENV_NULL=123456 hakoniwa run --config ./tests/config/field-envs.toml -- env
...
ENV_NULL=123456
...

```

## CfgEnv#value `""`

```console
$ ENV_BLANK=123456 hakoniwa run --config ./tests/config/field-envs.toml -- env
...
ENV_BLANK=
...

```
