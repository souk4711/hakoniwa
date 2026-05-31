# Installation

## From Releases

```sh
curl -fsSL https://raw.githubusercontent.com/souk4711/hakoniwa/refs/heads/main/install.sh | bash
```

## From Source

Run Cargo as your normal user. Only use `sudo` for package-manager commands,
privileged configuration, and copying the already-built `hakoniwa` binary into
`/usr/bin`.

### Arch, Manjaro and EndeavourOS based distributions

```sh
# Install dependencies
sudo pacman -S --noconfirm libseccomp passt shadow cargo

# Compile binary from source code as an unprivileged user
export RUSTUP_TOOLCHAIN=stable
install_root="$(mktemp -d)"
cargo install hakoniwa-cli --root "$install_root" --locked

# Install the already-built binary to /usr/bin/hakoniwa
sudo install -Dm755 "$install_root/bin/hakoniwa" /usr/bin/hakoniwa
```

### Debian and Ubuntu based distributions

```sh
# Install dependencies
sudo apt install -y libseccomp-dev passt uidmap cargo

# Compile binary from source code as an unprivileged user
install_root="$(mktemp -d)"
cargo install hakoniwa-cli --root "$install_root" --locked

# Install the already-built binary to /usr/bin/hakoniwa
sudo install -Dm755 "$install_root/bin/hakoniwa" /usr/bin/hakoniwa

# Configure AppArmor
apparmor_profile="$(mktemp)"
curl -o "$apparmor_profile" https://raw.githubusercontent.com/souk4711/hakoniwa/refs/heads/main/etc/apparmor.d/hakoniwa
sudo install -Dm644 "$apparmor_profile" /etc/apparmor.d/hakoniwa
sudo systemctl reload apparmor.service
```

### RHEL, Fedora and Rocky based distributions

```sh
# Install dependencies
sudo dnf install -y libseccomp-devel passt shadow-utils cargo

# Compile binary from source code as an unprivileged user
install_root="$(mktemp -d)"
cargo install hakoniwa-cli --root "$install_root" --locked

# Install the already-built binary to /usr/bin/hakoniwa
sudo install -Dm755 "$install_root/bin/hakoniwa" /usr/bin/hakoniwa

# Configure SELinux
sudo dnf install -y container-selinux
sudo chcon -u system_u -t container_runtime_exec_t /usr/bin/hakoniwa
```

### openSUSE based distributions

```sh
# Install dependencies
sudo zypper install -y libseccomp-devel passt shadow cargo

# Compile binary from source code as an unprivileged user
install_root="$(mktemp -d)"
cargo install hakoniwa-cli --root "$install_root" --locked

# Install the already-built binary to /usr/bin/hakoniwa
sudo install -Dm755 "$install_root/bin/hakoniwa" /usr/bin/hakoniwa
```
