# Template Renderer

## VAR

```console
$ MYENV=123 hakoniwa run --config ./tests/fixtures/config/cfg-template.toml -- env
...
MYENV=MYENV_123
...
```

## VAR `NULL`

```console
$ hakoniwa run --config ./tests/fixtures/config/cfg-template.toml -- env
...
MYENV=MYENV_
...
```
