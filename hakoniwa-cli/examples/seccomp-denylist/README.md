# Seccomp - denylist mode


## denylist mode

```sh
# Wget
$ hakoniwa run --policy-file ./policy.toml --verbose -- wget example.com
[2023-11-03T14:54:20Z INFO  hakoniwa::cli::run] Configuration: "./policy.toml"
[2023-11-03T14:54:20Z INFO  hakoniwa::executor] Mount point: host_path: "/tmp/hakoniwa-iMZRquU0", container_path: "/"
[2023-11-03T14:54:20Z INFO  hakoniwa::executor] Mount point: host_path: "", container_path: "/proc", fstype: "proc"
[2023-11-03T14:54:20Z INFO  hakoniwa::executor] Mount point: host_path: "/usr/bin", container_path: "/bin", fstype: "", rw: false
[2023-11-03T14:54:20Z INFO  hakoniwa::executor] Mount point: host_path: "/usr/lib", container_path: "/lib", fstype: "", rw: false
[2023-11-03T14:54:20Z INFO  hakoniwa::executor] Mount point: host_path: "/usr/lib", container_path: "/lib64", fstype: "", rw: false
[2023-11-03T14:54:20Z INFO  hakoniwa::executor] Mount point: host_path: "/usr", container_path: "/usr", fstype: "", rw: false
[2023-11-03T14:54:20Z INFO  hakoniwa::executor] Mount point: host_path: "/etc/resolv.conf", container_path: "/etc/resolv.conf", fstype: "", rw: false
[2023-11-03T14:54:20Z INFO  hakoniwa::executor] UID map: host_id: 1000, container_id: 1000
[2023-11-03T14:54:20Z INFO  hakoniwa::executor] GID map: host_id: 1000, container_id: 1000
[2023-11-03T14:54:20Z INFO  hakoniwa::executor] Seccomp: enabled (syscalls: 1): listen
[2023-11-03T14:54:20Z INFO  hakoniwa::executor] Seccomp: use `sudo ausearch -ts 22:54:20 -m seccomp -i` to know more detail
[2023-11-03T14:54:20Z INFO  hakoniwa::executor] Execve: /usr/bin/wget ["wget", "example.com"]
ERROR: could not open HSTS store. HSTS will be disabled.
--2023-11-03 14:54:20--  http://example.com/
Resolving example.com (example.com)... 2606:2800:220:1:248:1893:25c8:1946, 93.184.216.34
Connecting to example.com (example.com)|2606:2800:220:1:248:1893:25c8:1946|:80... connected.
HTTP request sent, awaiting response... 200 OK
Length: 1256 (1.2K) [text/html]
Saving to: 'index.html'

index.html                             100%[============================================================================>]   1.23K  --.-KB/s    in 0s

2023-11-03 14:54:20 (393 MB/s) - 'index.html' saved [1256/1256]

[2023-11-03T14:54:20Z INFO  hakoniwa::executor] Result: {"status":"OK","reason":"","exit_code":0,"start_time":"2023-11-03T14:54:20.337826605Z","real_time":{"secs":0,"nanos":389580338},"system_time":{"secs":0,"nanos":0},"user_time":{"secs":0,"nanos":4142000},"max_rss":5632}

# Python HTTP Server
$ hakoniwa run --policy-file ./policy.toml --verbose -- /usr/bin/python -m http.server 8000
[2023-11-03T14:55:30Z INFO  hakoniwa::cli::run] Configuration: "./policy.toml"
[2023-11-03T14:55:30Z INFO  hakoniwa::executor] Mount point: host_path: "/tmp/hakoniwa-ogdxH2PO", container_path: "/"
[2023-11-03T14:55:30Z INFO  hakoniwa::executor] Mount point: host_path: "", container_path: "/proc", fstype: "proc"
[2023-11-03T14:55:30Z INFO  hakoniwa::executor] Mount point: host_path: "/usr/bin", container_path: "/bin", fstype: "", rw: false
[2023-11-03T14:55:30Z INFO  hakoniwa::executor] Mount point: host_path: "/usr/lib", container_path: "/lib", fstype: "", rw: false
[2023-11-03T14:55:30Z INFO  hakoniwa::executor] Mount point: host_path: "/usr/lib", container_path: "/lib64", fstype: "", rw: false
[2023-11-03T14:55:30Z INFO  hakoniwa::executor] Mount point: host_path: "/usr", container_path: "/usr", fstype: "", rw: false
[2023-11-03T14:55:30Z INFO  hakoniwa::executor] Mount point: host_path: "/etc/resolv.conf", container_path: "/etc/resolv.conf", fstype: "", rw: false
[2023-11-03T14:55:30Z INFO  hakoniwa::executor] UID map: host_id: 1000, container_id: 1000
[2023-11-03T14:55:30Z INFO  hakoniwa::executor] GID map: host_id: 1000, container_id: 1000
[2023-11-03T14:55:30Z INFO  hakoniwa::executor] Seccomp: enabled (syscalls: 1): listen
[2023-11-03T14:55:30Z INFO  hakoniwa::executor] Seccomp: use `sudo ausearch -ts 22:55:30 -m seccomp -i` to know more detail
[2023-11-03T14:55:30Z INFO  hakoniwa::executor] Execve: /usr/bin/python ["/usr/bin/python", "-m", "http.server", "8000"]
[2023-11-03T14:55:30Z INFO  hakoniwa::executor] Result: {"status":"RFE","reason":"signaled: SIGSYS","exit_code":159,"start_time":"2023-11-03T14:55:30.463409457Z","real_time":{"secs":0,"nanos":232456304},"system_time":{"secs":0,"nanos":7282000},"user_time":{"secs":0,"nanos":32892000},"max_rss":18304}
```
