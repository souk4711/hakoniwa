# Howto - Track all syscalls invoked in realtime


## auditd

Start `audit` service:

```sh
$ sudo systemctl start auditd
```

Add following section to your sandbox policy configuration:

```toml
[seccomp]
dismatch_action = "log"
syscalls = []
```

Use `--verbose` flag to run the COMMAND:

```sh
$ hakoniwa run --policy-file ./policy.toml --verbose -- echo
...
[2022-08-22T09:15:56Z INFO  hakoniwa::executor] Seccomp: enabled (syscalls: 0):
[2022-08-22T09:15:56Z INFO  hakoniwa::executor] Seccomp: use `sudo ausearch -ts 17:15:56 -m seccomp -i` to know more detail
[2022-08-22T09:15:56Z INFO  hakoniwa::executor] Execve: /usr/bin/echo ["echo"]
...
```

Copy `sudo ausearch ...` and run it:

```sh
$ sudo ausearch -ts 17:15:56 -m seccomp -i
----
type=SECCOMP msg=audit(08/22/2022 17:15:56.273:401) : auid=johndoe uid=johndoe gid=johndoe ses=1 pid=3443 comm=hakoniwa exe=/usr/bin/hakoniwa sig=SIG0 arch=x86_64 syscall=execve compat=0 ip=0x7fb5638f2d1b code=log
----
type=SECCOMP msg=audit(08/22/2022 17:15:56.273:402) : auid=johndoe uid=johndoe gid=johndoe ses=1 pid=3443 comm=echo exe=/usr/bin/echo sig=SIG0 arch=x86_64 syscall=brk compat=0 ip=0x7f20657f380b code=log
...
----
type=SECCOMP msg=audit(08/22/2022 17:15:56.273:468) : auid=johndoe uid=johndoe gid=johndoe ses=1 pid=3443 comm=echo exe=/usr/bin/echo sig=SIG0 arch=x86_64 syscall=close compat=0 ip=0x7f20656e5b0b code=log
----
type=SECCOMP msg=audit(08/22/2022 17:15:56.273:469) : auid=johndoe uid=johndoe gid=johndoe ses=1 pid=3443 comm=echo exe=/usr/bin/echo sig=SIG0 arch=x86_64 syscall=close compat=0 ip=0x7f20656e5b0b code=log
----
type=SECCOMP msg=audit(08/22/2022 17:15:56.273:470) : auid=johndoe uid=johndoe gid=johndoe ses=1 pid=3443 comm=echo exe=/usr/bin/echo sig=SIG0 arch=x86_64 syscall=exit_group compat=0 ip=0x7f20656bc661 code=log
```

To summarize:

```sh
$ sudo ausearch -ts 17:15:56 -m seccomp -i | awk -F " : " '{ print $2 }' | awk -F "[ =]" '{ print $20 }' | sort | uniq

access
arch_prctl
brk
close
execve
exit_group
getrandom
mmap
mprotect
newfstatat
openat
pread
prlimit64
read
rseq
set_robust_list
set_tid_address
write
```
