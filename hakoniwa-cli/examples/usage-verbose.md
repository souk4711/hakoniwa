# Usage - Verbose Output


## --verbose

Use verbose output

```sh
$ hakoniwa run --verbose -- echo "Hako!"
[2022-08-12T07:56:53Z INFO  hakoniwa::cli::run] Configuration: "KISS-policy.toml"
[2022-08-12T07:56:53Z INFO  hakoniwa::executor] Mount point: host_path: "/tmp/hakoniwa-esM3q6P2", container_path: "/"
[2022-08-12T07:56:53Z INFO  hakoniwa::executor] Mount point: host_path: none, container_path: "/proc", fstype: proc
[2022-08-12T07:56:53Z INFO  hakoniwa::executor] Mount point: host_path: none, container_path: "/tmp", fstype: tmpfs
[2022-08-12T07:56:53Z INFO  hakoniwa::executor] Mount point: host_path: "/dev/null", container_path: "/dev/null"
[2022-08-12T07:56:53Z INFO  hakoniwa::executor] Mount point: host_path: "/dev/random", container_path: "/dev/random"
[2022-08-12T07:56:53Z INFO  hakoniwa::executor] Mount point: host_path: "/dev/urandom", container_path: "/dev/urandom"
[2022-08-12T07:56:53Z INFO  hakoniwa::executor] Mount point: host_path: "/dev/zero", container_path: "/dev/zero"
[2022-08-12T07:56:53Z INFO  hakoniwa::executor] Mount point: host_path: "/usr/bin", container_path: "/bin", readonly: true
[2022-08-12T07:56:53Z INFO  hakoniwa::executor] Mount point: host_path: "/usr/lib", container_path: "/lib", readonly: true
[2022-08-12T07:56:53Z INFO  hakoniwa::executor] Mount point: host_path: "/usr/lib", container_path: "/lib64", readonly: true
[2022-08-12T07:56:53Z INFO  hakoniwa::executor] Mount point: host_path: "/usr", container_path: "/usr", readonly: true
[2022-08-12T07:56:53Z INFO  hakoniwa::executor] UID map: host_id: 5001, container_id: 5001
[2022-08-12T07:56:53Z INFO  hakoniwa::executor] GID map: host_id: 1000, container_id: 1000
[2022-08-12T07:56:53Z INFO  hakoniwa::executor] Seccomp: disabled
[2022-08-12T07:56:53Z INFO  hakoniwa::executor] Execve: /usr/bin/echo ["echo", "Hako!"]
Hako!
[2022-08-12T07:56:53Z INFO  hakoniwa::executor] Result: {"status":"OK","reason":"","exit_code":0,"start_time":"2022-08-12T07:56:53.235900544Z","real_time":{"secs":0,"nanos":1461176},"system_time":{"secs":0,"nanos":0},"user_time":{"secs":0,"nanos":1365000},"max_rss":1464}
```
