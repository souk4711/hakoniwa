# Seccomp - allowlist mode


## allowlist mode

```console
$ hakoniwa run --policy-file ./policy.toml --verbose -- /usr/bin/echo "Hako!"
[2023-11-03T14:40:16Z INFO  hakoniwa::cli::run] Configuration: "./policy.toml"
[2023-11-03T14:40:16Z INFO  hakoniwa::executor] Mount point: host_path: "/tmp/hakoniwa-hyEJAvtS", container_path: "/"
[2023-11-03T14:40:16Z INFO  hakoniwa::executor] Mount point: host_path: "", container_path: "/proc", fstype: "proc"
[2023-11-03T14:40:16Z INFO  hakoniwa::executor] Mount point: host_path: "/usr/bin", container_path: "/bin", fstype: "", rw: false
[2023-11-03T14:40:16Z INFO  hakoniwa::executor] Mount point: host_path: "/usr/lib", container_path: "/lib", fstype: "", rw: false
[2023-11-03T14:40:16Z INFO  hakoniwa::executor] Mount point: host_path: "/usr/lib", container_path: "/lib64", fstype: "", rw: false
[2023-11-03T14:40:16Z INFO  hakoniwa::executor] Mount point: host_path: "/usr", container_path: "/usr", fstype: "", rw: false
[2023-11-03T14:40:16Z INFO  hakoniwa::executor] UID map: host_id: 1000, container_id: 1000
[2023-11-03T14:40:16Z INFO  hakoniwa::executor] GID map: host_id: 1000, container_id: 1000
[2023-11-03T14:40:16Z INFO  hakoniwa::executor] Seccomp: enabled (syscalls: 20): access,arch_prctl,brk,close,execve,exit_group,getrandom,ioctl,mmap,mprotect,munmap,newfstatat,openat,pread64,prlimit64,read,rseq,set_robust_list,set_tid_address,write
[2023-11-03T14:40:16Z INFO  hakoniwa::executor] Seccomp: use `sudo ausearch -ts 22:40:16 -m seccomp -i` to know more detail
[2023-11-03T14:40:16Z INFO  hakoniwa::executor] Execve: /usr/bin/echo ["/usr/bin/echo", "Hako!"]
Hako!
[2023-11-03T14:40:16Z INFO  hakoniwa::executor] Result: {"status":"OK","reason":"","exit_code":0,"start_time":"2023-11-03T14:40:16.937964608Z","real_time":{"secs":0,"nanos":753853},"system_time":{"secs":0,"nanos":0},"user_time":{"secs":0,"nanos":708000},"max_rss":3312}
```
