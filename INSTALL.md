# Installation

## From Releases

```sh
curl --proto '=https' --tlsv1.2 -fsSL https://raw.githubusercontent.com/souk4711/hakoniwa/refs/heads/main/install.sh | bash
```

## From Source

> [!TIP]
> Run `cargo` as your normal user. Only use `sudo` for package-manager commands,
> privileged configuration, and copying the already-built `hakoniwa` binary into
> `/usr/bin`.

### Arch, Manjaro and EndeavourOS based distributions

```sh
# Install dependencies
sudo pacman -S --noconfirm libseccomp passt shadow cargo

# Compile binary from source code as an unprivileged user
tmp_install="$(mktemp -d)"
cargo install hakoniwa-cli --root "$tmp_install" --locked

# Install the already-built binary to /usr/bin/hakoniwa
sudo install -Dm755 "$tmp_install/bin/hakoniwa" /usr/bin/hakoniwa
```

### Debian and Ubuntu based distributions

```sh
# Install dependencies
sudo apt install -y libseccomp-dev passt uidmap cargo

# Compile binary from source code as an unprivileged user
tmp_install="$(mktemp -d)"
cargo install hakoniwa-cli --root "$tmp_install" --locked

# Install the already-built binary to /usr/bin/hakoniwa
sudo install -Dm755 "$tmp_install/bin/hakoniwa" /usr/bin/hakoniwa

# Configure AppArmor
sudo tee /etc/apparmor.d/hakoniwa >/dev/null <<'EOF'
# This profile allows everything and only exists to give the
# application a name instead of having the label "unconfined"

abi <abi/4.0>,
include <tunables/global>

profile hakoniwa /usr/bin/hakoniwa flags=(unconfined) {
  userns,

  # Site-specific additions and overrides. See local/README for details.
  include if exists <local/hakoniwa>
}
EOF
sudo systemctl reload apparmor.service
```

### RHEL, Fedora and Rocky based distributions

```sh
# Install dependencies
sudo dnf install -y libseccomp-devel passt shadow-utils cargo

# Compile binary from source code as an unprivileged user
tmp_install="$(mktemp -d)"
cargo install hakoniwa-cli --root "$tmp_install" --locked

# Install the already-built binary to /usr/bin/hakoniwa
sudo install -Dm755 "$tmp_install/bin/hakoniwa" /usr/bin/hakoniwa

# Configure SELinux
sudo dnf install -y container-selinux
sudo chcon -u system_u -t container_runtime_exec_t /usr/bin/hakoniwa
```

### openSUSE based distributions

```sh
# Install dependencies
sudo zypper install -y libseccomp-devel passt shadow cargo

# Compile binary from source code as an unprivileged user
tmp_install="$(mktemp -d)"
cargo install hakoniwa-cli --root "$tmp_install" --locked

# Install the already-built binary to /usr/bin/hakoniwa
sudo install -Dm755 "$tmp_install/bin/hakoniwa" /usr/bin/hakoniwa
```
