# Lang - C++


## G++ with static linking

```sh
# Compile
$ hakoniwa run --setenv PATH=$PATH --work-dir . -- /usr/bin/g++ --static main.cpp -o main

# Run
$ hakoniwa run --policy-file ./policy.toml --ro-bind ./main:/bin/main -- /bin/main
Hello, World!
```
