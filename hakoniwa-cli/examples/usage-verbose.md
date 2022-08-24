# Usage - Verbose Output


## --verbose

Use verbose output

```sh
$ hakoniwa run --verbose -- echo "Hako!"
[2022-08-21T10:17:39Z INFO  hakoniwa::cli::run] Configuration: "KISS-policy.toml"
[2022-08-21T10:17:39Z INFO  hakoniwa::executor] Mount point: host_path: "/tmp/hakoniwa-s8wvt60l", container_path: "/"
[2022-08-21T10:17:39Z INFO  hakoniwa::executor] Mount point: host_path: "", container_path: "/proc", fstype: "proc"
[2022-08-21T10:17:39Z INFO  hakoniwa::executor] Mount point: host_path: "/usr/bin", container_path: "/bin", fstype: "", rw: false
[2022-08-21T10:17:39Z INFO  hakoniwa::executor] Mount point: host_path: "/usr/lib", container_path: "/lib", fstype: "", rw: false
[2022-08-21T10:17:39Z INFO  hakoniwa::executor] Mount point: host_path: "/usr/lib", container_path: "/lib64", fstype: "", rw: false
[2022-08-21T10:17:39Z INFO  hakoniwa::executor] Mount point: host_path: "/usr", container_path: "/usr", fstype: "", rw: false
[2022-08-21T10:17:39Z INFO  hakoniwa::executor] Mount point: host_path: "/dev/null", container_path: "/dev/null", fstype: "", rw: true
[2022-08-21T10:17:39Z INFO  hakoniwa::executor] Mount point: host_path: "/dev/random", container_path: "/dev/random", fstype: "", rw: false
[2022-08-21T10:17:39Z INFO  hakoniwa::executor] Mount point: host_path: "/dev/urandom", container_path: "/dev/urandom", fstype: "", rw: false
[2022-08-21T10:17:39Z INFO  hakoniwa::executor] Mount point: host_path: "/dev/zero", container_path: "/dev/zero", fstype: "", rw: false
[2022-08-21T10:17:39Z INFO  hakoniwa::executor] UID map: host_id: 5001, container_id: 5001
[2022-08-21T10:17:39Z INFO  hakoniwa::executor] GID map: host_id: 1000, container_id: 1000
[2022-08-21T10:17:39Z INFO  hakoniwa::executor] Seccomp: disabled
[2022-08-21T10:17:39Z INFO  hakoniwa::executor] Execve: /usr/bin/echo ["echo", "Hako!"]
Hako!
[2022-08-21T10:17:39Z INFO  hakoniwa::executor] Result: {"status":"OK","reason":"","exit_code":0,"start_time":"2022-08-21T10:17:39.760729871Z","real_time":{"secs":0,"nanos":1351771},"system_time":{"secs":0,"nanos":1260000},"user_time":{"secs":0,"nanos":0},"max_rss":1416}
```
