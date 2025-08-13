# Installation

## Pre-compiled Binary

1. Install dependencies:
   - [libseccomp](https://github.com/seccomp/libseccomp)
   - [passt](https://passt.top/passt/about/)
   - [shadow](https://github.com/shadow-maint/shadow)

2. Download a pre-compiled binary from [Releases](https://github.com/souk4711/hakoniwa/releases).

3. Configure [AppArmor](./hakoniwa-cli/docs/troubleshooting-apparmor) or SELinux, if enabled.

## From Source

1. Install dependencies:
   - [libseccomp](https://github.com/seccomp/libseccomp)
   - [passt](https://passt.top/passt/about/)
   - [shadow](https://github.com/shadow-maint/shadow)

2. Compile binary from source code and install to `/usr/bin/hakoniwa`:

   ```sh
   cargo install hakoniwa-cli --root /usr --git https://github.com/souk4711/hakoniwa.git --locked
   ```

3. Configure [AppArmor](./hakoniwa-cli/docs/troubleshooting-apparmor) or SELinux, if enabled.

## Distros

### Arch, Manjaro and EndeavourOS based distributions

```sh
# Install dependencies
pacman -S --noconfirm libseccomp passt shadow cargo

# Compile binary from source code and install to /usr/bin/hakoniwa
cargo install hakoniwa-cli --root /usr --locked
```

### Debian and Ubuntu based distributions

```sh
# Install dependencies
apt install -y libseccomp-dev passt uidmap cargo

# Compile binary from source code and install to /usr/bin/hakoniwa
cargo install hakoniwa-cli --root /usr --locked

# Configure AppArmor
curl -o /etc/apparmor.d/hakoniwa https://raw.githubusercontent.com/souk4711/hakoniwa/refs/heads/main/etc/apparmor.d/hakoniwa
systemctl reload apparmor.service
```

### RHEL, Fedora and Rocky based distributions

```sh
# Install dependencies
dnf install -y libseccomp-devel passt shadow-utils cargo

# Compile binary from source code and install to /usr/bin/hakoniwa
cargo install hakoniwa-cli --root /usr --locked

# Configure SELinux
dnf install -y container-selinux
chcon -u system_u -t container_runtime_exec_t /usr/bin/hakoniwa
```
