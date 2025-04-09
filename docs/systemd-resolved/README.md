# systemd-resolved

## Problem

On some distros, they use `systemd-resolved` to resolve domain names. It will rewrite your
DNS to point to `127.0.0.53` by default, which is not reachable inside the sandboxed program
if you start it with `--unshare-network` or `--network=pasta`.

```console,ignore
johndoe@ubuntu:~$ ls -l /etc/resolv.conf
lrwxrwxrwx 1 root root 39 3月  30 03:52 /etc/resolv.conf -> ../run/systemd/resolve/stub-resolv.conf

johndoe@ubuntu:~$ cat /etc/resolv.conf
nameserver 127.0.0.53
options edns0 trust-ad
search lan

johndoe@ubuntu:~$ hakoniwa run --unshare-all --network=pasta -- wget https://example.com --spider
Spider mode enabled. Check if remote file exists.
--2025-04-09 02:18:54--  https://example.com/
Resolving example.com (example.com)... failed: Temporary failure in name resolution.
wget: unable to resolve host address 'example.com'
```

## Solution

### 1. bind mount `/run/systemd/resolve/stub-resolv.conf`

If you start sandboxed program with `--rootfs=/`, then you will not be able to mount `/etc/resolv.conf` due
to lack of permission, but you can still mount a file in the `/run` folder:

```sh
hakoniwa run --unshare-all --network=pasta \
  -b /run/systemd/resolve/resolv.conf:/run/systemd/resolve/stub-resolv.conf \
  -- wget https://example.com --spider
```

If you start sandboxed program with `--rootfs=none`, then just mount `/etc/resolv.conf`:

```sh
hakoniwa run --unshare-all --network=pasta \
  --rootfs=none -b /bin -b /lib -b /lib64 -b /usr \
  -b /run/systemd/resolve/resolv.conf:/etc/resolv.conf \
  -- wget https://example.com --spider
```

### 2. run with `network=host`

Another solution is use `host` network:

```sh
hakoniwa run --unshare-all --network=host \
  -b /run/systemd/resolve/ \
  -- wget https://example.com --spider
```

### 3. app-specify DNS configuration

Some CLI tools can specify DNS server through arguments, e.g.:

```sh
hakoniwa run --unshare-all --network=pasta \
  -- aria2c https://example.com --async-dns-server=8.8.8.8 --dry-run
```

For most browsers (e.g. Firefox), you can use DNS over HTTPS, read [this][Configure DoH on your browser] to learn more.

## Links

- [systemd-resolved - ArchWiki]
- [aria2c#cmdoption-async-dns-server]
- [Configure DoH on your browser]

[aria2c#cmdoption-async-dns-server]: https://aria2.github.io/manual/en/html/aria2c.html#cmdoption-async-dns-server
[systemd-resolved - ArchWiki]: https://wiki.archlinux.org/title/Systemd-resolved
[Configure DoH on your browser]: https://developers.cloudflare.com/1.1.1.1/encryption/dns-over-https/encrypted-dns-browsers/
