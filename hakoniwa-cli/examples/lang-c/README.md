# Lang - C

## GCC with static linking

```sh
# Compile
$ hakoniwa run --setenv PATH=$PATH --work-dir . -- gcc main.c -o main --static

# Run
$ hakoniwa run --policy-file ./policy.toml --ro-bind $PWD/main:/bin/main -- /bin/main
Hello, World!
```
