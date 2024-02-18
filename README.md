# Hakoniwa

Process isolation for Linux using namespaces, resource limits and seccomp. It
works by creating a new, completely empty, mount namespace where the root is
on a tmpfs that is invisible from the host, and will be automatically cleaned
up when the last process exits. You can then use a policy configuration file or
commandline options to construct the root filesystem and process environment
and command to run in the namespace.


## Installation

### Cargo

* Install libseccomp by following [this][Install libseccomp] guide.
* Install the rust toolchain in order to have cargo installed by following
  [this][Install Rust] guide.
* Run `cargo install hakoniwa-cli`.


## Usage

### CLI

When use commandline, `hakoniwa-run` will load a default policy configuration named
[KISS-policy.toml] to ensure a minimal mount namespace created, use `--policy-file`
to use your custom version.

```console
$ hakoniwa run --verbose -- /bin/bash
[2022-08-21T09:14:11Z INFO  hakoniwa::cli::run] Configuration: "KISS-policy.toml"
[2022-08-21T09:14:11Z INFO  hakoniwa::executor] Mount point: host_path: "/tmp/hakoniwa-EJemcsRL", container_path: "/"
[2022-08-21T09:14:11Z INFO  hakoniwa::executor] Mount point: host_path: "", container_path: "/proc", fstype: "proc"
[2022-08-21T09:14:11Z INFO  hakoniwa::executor] Mount point: host_path: "/usr/bin", container_path: "/bin", fstype: "", rw: false
[2022-08-21T09:14:11Z INFO  hakoniwa::executor] Mount point: host_path: "/usr/lib", container_path: "/lib", fstype: "", rw: false
[2022-08-21T09:14:11Z INFO  hakoniwa::executor] Mount point: host_path: "/usr/lib", container_path: "/lib64", fstype: "", rw: false
[2022-08-21T09:14:11Z INFO  hakoniwa::executor] Mount point: host_path: "/usr", container_path: "/usr", fstype: "", rw: false
[2022-08-21T09:14:11Z INFO  hakoniwa::executor] Mount point: host_path: "/dev/null", container_path: "/dev/null", fstype: "", rw: true
[2022-08-21T09:14:11Z INFO  hakoniwa::executor] Mount point: host_path: "/dev/random", container_path: "/dev/random", fstype: "", rw: true
[2022-08-21T09:14:11Z INFO  hakoniwa::executor] Mount point: host_path: "/dev/urandom", container_path: "/dev/urandom", fstype: "", rw: true
[2022-08-21T09:14:11Z INFO  hakoniwa::executor] Mount point: host_path: "/dev/zero", container_path: "/dev/zero", fstype: "", rw: true
[2022-08-21T09:14:11Z INFO  hakoniwa::executor] UID map: host_id: 5001, container_id: 5001
[2022-08-21T09:14:11Z INFO  hakoniwa::executor] GID map: host_id: 1000, container_id: 1000
[2022-08-21T09:14:11Z INFO  hakoniwa::executor] Seccomp: disabled
[2022-08-21T09:14:11Z INFO  hakoniwa::executor] Execve: /bin/bash ["/bin/bash"]
bash: cannot set terminal process group (-1): Inappropriate ioctl for device
bash: no job control in this shell
bash-5.1$ pwd
/
bash-5.1$ ls
bin  dev  lib  lib64  proc  usr
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
[2022-08-21T09:14:27Z INFO  hakoniwa::executor] Result: {"status":"OK","reason":"","exit_code":0,"start_time":"2022-08-21T09:14:11.058546277Z","real_time":{"secs":16,"nanos":460452556},"system_time":{"secs":0,"nanos":8744000},"user_time":{"secs":0,"nanos":3149000},"max_rss":3780}
```

More examples can be found in [hakoniwa-cli/examples].

### Rust Library

The code below is almost eq to `hakoniwa run --policy-file KISS-policy.toml -- /bin/bash`:

```rust
use hakoniwa::{Error, Sandbox, SandboxPolicy, Stdio};

fn main() -> Result<(), Error> {
    let policy = SandboxPolicy::from_str(
        r#"
mounts = [
  { source = "/bin"        , target = "/bin"         },
  { source = "/lib"        , target = "/lib"         },
  { source = "/lib64"      , target = "/lib64"       },
  { source = "/usr"        , target = "/usr"         },
  { source = "/dev/null"   , target = "/dev/null"     , rw = true },
  { source = "/dev/random" , target = "/dev/random"   , rw = true },
  { source = "/dev/urandom", target = "/dev/urandom"  , rw = true },
  { source = "/dev/zero"   , target = "/dev/zero"     , rw = true },
]

[env]
LANG     = {{ os_env "LANG"     }}
LANGUAGE = {{ os_env "LANGUAGE" }}
LC_ALL   = {{ os_env "LC_ALL"   }}
TERM     = {{ os_env "TERM"     }}
    "#,
    )?;

    let mut sandbox = Sandbox::new();
    sandbox.with_policy(policy);

    let prog = std::env::var("SHELL").unwrap_or_else(|_| String::from("/bin/sh"));
    let argv = vec![&prog];
    let mut executor = sandbox.command(&prog, &argv);
    let result = executor
        // .ro_bind("/etc", "/myetc")? // --ro-bind /etc:/myetc
        // .rw_bind("/data", "/data")? // --rw-bind /data
        // .limit_cpu(Some(2)) // --limit-cpu 2
        // .limit_walltime(Some(5)) // --limit-walltime 5
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .stdin(Stdio::inherit())
        .run();

    dbg!(result);
    Ok(())
}
```

More examples can be found in [hakoniwa/examples].

### Running inside Docker

First, clone this repository and build the docker image:

```console
$ make prodcontainer
```

Then, run `hakoniwa` command in the container:

```console
$ docker run --privileged --group-add keep-groups --rm -it hakoniwa-prodcontainer:latest hakoniwa run --verbose -- /bin/bash
[2023-11-04T09:24:27Z INFO  hakoniwa::cli::run] Configuration: "KISS-policy.toml"
[2023-11-04T09:24:27Z INFO  hakoniwa::executor] Mount point: host_path: "/tmp/hakoniwa-yBV2slf6", container_path: "/"
[2023-11-04T09:24:27Z INFO  hakoniwa::executor] Mount point: host_path: "", container_path: "/proc", fstype: "proc"
[2023-11-04T09:24:27Z INFO  hakoniwa::executor] Mount point: host_path: "/bin", container_path: "/bin", fstype: "", rw: false
[2023-11-04T09:24:27Z INFO  hakoniwa::executor] Mount point: host_path: "/lib", container_path: "/lib", fstype: "", rw: false
[2023-11-04T09:24:27Z INFO  hakoniwa::executor] Mount point: host_path: "/lib64", container_path: "/lib64", fstype: "", rw: false
[2023-11-04T09:24:27Z INFO  hakoniwa::executor] Mount point: host_path: "/usr", container_path: "/usr", fstype: "", rw: false
[2023-11-04T09:24:27Z INFO  hakoniwa::executor] Mount point: host_path: "/dev/null", container_path: "/dev/null", fstype: "", rw: true
[2023-11-04T09:24:27Z INFO  hakoniwa::executor] Mount point: host_path: "/dev/random", container_path: "/dev/random", fstype: "", rw: true
[2023-11-04T09:24:27Z INFO  hakoniwa::executor] Mount point: host_path: "/dev/urandom", container_path: "/dev/urandom", fstype: "", rw: true
[2023-11-04T09:24:27Z INFO  hakoniwa::executor] Mount point: host_path: "/dev/zero", container_path: "/dev/zero", fstype: "", rw: true
[2023-11-04T09:24:27Z INFO  hakoniwa::executor] UID map: host_id: 1000, container_id: 1000
[2023-11-04T09:24:27Z INFO  hakoniwa::executor] GID map: host_id: 1000, container_id: 1000
[2023-11-04T09:24:27Z INFO  hakoniwa::executor] Seccomp: disabled
[2023-11-04T09:24:27Z INFO  hakoniwa::executor] Execve: /bin/bash ["/bin/bash"]
bash: cannot set terminal process group (-1): Inappropriate ioctl for device
bash: no job control in this shell
bash-5.1$ pwd
/
bash-5.1$ ls
bin  dev  lib  lib64  proc  usr
bash-5.1$ ls /dev
null  random  urandom  zero
bash-5.1$ ls /proc
1           bus        crypto         execdomains  ioports    kmsg           locks    mtrr          scsi      sys            uptime
4           cgroups    devices        fb           irq        kpagecgroup    meminfo  net           self      sysrq-trigger  version
acpi        cmdline    diskstats      filesystems  kallsyms   kpagecount     misc     pagetypeinfo  slabinfo  sysvipc        vmallocinfo
asound      config.gz  dma            fs           kcore      kpageflags     modules  partitions    softirqs  thread-self    vmstat
bootconfig  consoles   driver         interrupts   keys       latency_stats  mounts   pressure      stat      timer_list     zoneinfo
buddyinfo   cpuinfo    dynamic_debug  iomem        key-users  loadavg        mtd      schedstat     swaps     tty
bash-5.1$ exit
exit
[2023-11-04T09:24:40Z INFO  hakoniwa::executor] Result: {"status":"OK","reason":"","exit_code":0,"start_time":"2023-11-04T09:24:27.975208221Z","real_time":{"secs":12,"nanos":171313268},"system_time":{"secs":0,"nanos":2516000},"user_time":{"secs":0,"nanos":10995000},"max_rss":3584}
```


## Howto

* [Run simple X11 application](./hakoniwa-cli/examples/howto-xorg-apps.md)
* [Track all syscalls invoked in realtime](./hakoniwa-cli/examples/howto-seccomp-syscalls.md)


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


[Install libseccomp]:https://github.com/libseccomp-rs/libseccomp-rs#requirements
[Install Rust]:https://www.rust-lang.org/tools/install
[bubblewrap]:https://github.com/containers/bubblewrap
[KISS-policy.toml]:./hakoniwa-cli/src/embed/KISS-policy.toml
[hakoniwa-cli/examples]:./hakoniwa-cli/examples
[hakoniwa/examples]:./hakoniwa/examples
