# Template Renderer

## var definded

```console
$ MYENV=123 hakoniwa run --config ./tests/config/template.toml -- env
...
MYENV=MYENV_123
...
```
## var undefinded

```console
$ hakoniwa run --config ./tests/config/template.toml -- env
...
MYENV=MYENV_
...
```
