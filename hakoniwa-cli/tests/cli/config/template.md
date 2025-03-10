# Template Renderer

## var defined

```console
$ MYENV=123 hakoniwa run --config ./tests/config/template.toml -- env
...
MYENV=MYENV_123
...
```

## var undefined

```console
$ hakoniwa run --config ./tests/config/template.toml -- env
...
MYENV=MYENV_
...
```
