# Lang - Go

## Go

```sh
# Compile
$ hakoniwa run --setenv HOME=/ --work-dir . -- /usr/bin/go build main.go

# Run
$ hakoniwa run --policy-file ./policy.toml --ro-bind ./main:/bin/main -- /bin/main
Hello, World!
```
