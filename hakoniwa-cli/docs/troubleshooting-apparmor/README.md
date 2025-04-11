# Permission issue caused by AppArmor

## Problem

Hakoniwa runs as an unprivileged user and requires the **Linux namespaces** feature.
But this feature is restricted by `AppArmor` on some distros.

```console
$ hakoniwa run
hakoniwa: write("/proc/self/uid_map", ...) => Operation not permitted (os error 1)
```

## Solution

### 1. Temporarily disabling the restriction

To disable

```sh
sudo sysctl -w kernel.apparmor_restrict_unprivileged_userns=0
```

To enable

```sh
sudo sysctl -w kernel.apparmor_restrict_unprivileged_userns=1
```

### 2. Permanently disabling the restriction

Create an unconfined profile `/etc/apparmor.d/hakoniwa` with the following content:

```
# This profile allows everything and only exists to give the
# application a name instead of having the label "unconfined"

abi <abi/4.0>,
include <tunables/global>

profile hakoniwa /usr/bin/hakoniwa flags=(unconfined) {
  userns,

  # Site-specific additions and overrides. See local/README for details.
  include if exists <local/hakoniwa>
}
```

Reload profile:

```
sudo systemctl reload apparmor.service
```

## Links

- [Restricted unprivileged user namespaces are coming to Ubuntu 23.10](https://ubuntu.com/blog/ubuntu-23-10-restricted-unprivileged-user-namespaces)
- [AppArmor - Ubuntu Server documentation](https://documentation.ubuntu.com/server/how-to/security/apparmor/index.html)
- [unprivileged_userns_restriction](https://gitlab.com/apparmor/apparmor/-/wikis/unprivileged_userns_restriction)
