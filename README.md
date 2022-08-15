# Hakoniwa

Process isolation for Linux using namespaces, resource limits and seccomp. It
works by creating a new, completely empty, mount namespace where the root is
on a tmpfs that is invisible from the host, and will be automatically cleaned
up when the last process exits. You can then use a policy configuration file or
commandline options to construct the root filesystem and process environment
and command to run in the namespace.


## Installation

### Cargo

* Install the rust toolchain in order to have cargo installed by following
  [this][Install Rust] guide.
* Run `cargo install hakoniwa-cli`.


## Usage

### CLI

When use commandline, `hakoniwa` will load a default policy configuration named
[KISS-policy.toml] to ensure a minimal mount namespace created, use `--policy-file`
to use your custom version.

```sh
$ hakoniwa run --verbose -- /bin/bash
[2022-08-14T06:37:18Z INFO  hakoniwa::cli::run] Configuration: "KISS-policy.toml"
[2022-08-14T06:37:18Z INFO  hakoniwa::executor] Mount point: host_path: "/tmp/hakoniwa-sPIay4xI", container_path: "/"
[2022-08-14T06:37:18Z INFO  hakoniwa::executor] Mount point: host_path: none, container_path: "/proc", fstype: proc
[2022-08-14T06:37:18Z INFO  hakoniwa::executor] Mount point: host_path: none, container_path: "/tmp", fstype: tmpfs
[2022-08-14T06:37:18Z INFO  hakoniwa::executor] Mount point: host_path: "/dev/null", container_path: "/dev/null"
[2022-08-14T06:37:18Z INFO  hakoniwa::executor] Mount point: host_path: "/dev/random", container_path: "/dev/random"
[2022-08-14T06:37:18Z INFO  hakoniwa::executor] Mount point: host_path: "/dev/urandom", container_path: "/dev/urandom"
[2022-08-14T06:37:18Z INFO  hakoniwa::executor] Mount point: host_path: "/dev/zero", container_path: "/dev/zero"
[2022-08-14T06:37:18Z INFO  hakoniwa::executor] Mount point: host_path: "/usr/bin", container_path: "/bin", readonly: true
[2022-08-14T06:37:18Z INFO  hakoniwa::executor] Mount point: host_path: "/usr/lib", container_path: "/lib", readonly: true
[2022-08-14T06:37:18Z INFO  hakoniwa::executor] Mount point: host_path: "/usr/lib", container_path: "/lib64", readonly: true
[2022-08-14T06:37:18Z INFO  hakoniwa::executor] Mount point: host_path: "/usr", container_path: "/usr", readonly: true
[2022-08-14T06:37:18Z INFO  hakoniwa::executor] UID map: host_id: 5001, container_id: 5001
[2022-08-14T06:37:18Z INFO  hakoniwa::executor] GID map: host_id: 1000, container_id: 1000
[2022-08-14T06:37:18Z INFO  hakoniwa::executor] Seccomp: disabled
[2022-08-14T06:37:18Z INFO  hakoniwa::executor] Execve: /bin/bash ["/bin/bash"]
bash: cannot set terminal process group (-1): Inappropriate ioctl for device
bash: no job control in this shell
bash-5.1$ pwd
/
bash-5.1$ ls
bin  dev  lib  lib64  proc  tmp  usr
bash-5.1$ ls /dev
null  random  urandom  zero
bash-5.1$ ls /proc
1           bus        crypto         execdomains  ioports    kmsg           locks    mtrr          scsi      sys            uptime
4           cgroups    devices        fb           irq        kpagecgroup    meminfo  net           self      sysrq-trigger  version
acpi        cmdline    diskstats      filesystems  kallsyms   kpagecount     misc     pagetypeinfo  slabinfo  sysvipc        vmallocinfo
asound      config.gz  dma            fs           kcore      kpageflags     modules  partitions    softirqs  thread-self    vmstat
bootconfig  consoles   driver         interrupts   key-users  latency_stats  mounts   pressure      stat      timer_list     zoneinfo
buddyinfo   cpuinfo    dynamic_debug  iomem        keys       loadavg        mtd      schedstat     swaps     tty
bash-5.1$ exit
exit
[2022-08-14T06:37:30Z INFO  hakoniwa::executor] Result: {"status":"OK","reason":"","exit_code":0,"start_time":"2022-08-14T06:37:18.589010919Z","real_time":{"secs":12,"nanos":382268418},"system_time":{"secs":0,"nanos":6211000},"user_time":{"secs":0,"nanos":8138000},"max_rss":3748}
$
```

More examples can be found in [hakoniwa-cli/examples].

### Rust Library

The code below is almost eq to `hakoniwa run --policy-file KISS-policy.toml -- /bin/bash`:

```rust
extern crate hakoniwa;

use hakoniwa::{Error, Sandbox, SandboxPolicy};

fn main() -> Result<(), Error> {
    let policy = SandboxPolicy::from_str(
        r#"
mount_new_tmpfs = true
mount_new_devfs = true
mounts = [
  { source = "/bin"  , target = "/bin"  },
  { source = "/lib"  , target = "/lib"  },
  { source = "/lib64", target = "/lib64"},
  { source = "/usr"  , target = "/usr"  },
]

[env]
TERM = {{ os_env "TERM" }}
    "#,
    )?;

    let mut sandbox = Sandbox::new();
    sandbox.with_policy(policy);

    let prog = std::env::var("SHELL").unwrap_or_else(|_| String::from("/bin/sh"));
    let mut executor = sandbox.command(&prog, &[prog.as_str()]);
    executor
        // .ro_bind("/etc", "/myetc")? // --ro-bind /etc:/myetc
        // .bind("/data", "/data")? // --bind /data
        // .limit_cpu(Some(2)) // --limit-cpu 2
        // .limit_walltime(Some(5)) // --limit-walltime 5
        .run();

    Ok(())
}
```


## Acknowledgements

* Special thanks to [bubblewrap].


## License

Licensed under either of

* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.


## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.


[Install Rust]:https://www.rust-lang.org/tools/install
[bubblewrap]:https://github.com/containers/bubblewrap
[KISS-policy.toml]:https://github.com/souk4711/hakoniwa/blob/main/hakoniwa-cli/src/embed/KISS-policy.toml
[hakoniwa-cli/examples]:https://github.com/souk4711/hakoniwa/tree/main/hakoniwa-cli/examples
[hakoniwa/examples]:https://github.com/souk4711/hakoniwa/tree/main/hakoniwa/examples
