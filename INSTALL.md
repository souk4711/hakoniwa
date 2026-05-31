# Installation

## From Releases

Download the installer and checksums from the same immutable release tag you are
installing, verify `install.sh`, then run the installer with the pinned version and
the expected SHA-256 digest for your architecture from the release announcement.

```sh
version=v1.7.0
arch=$(uname -m)
case "$arch" in
  amd64|x86_64) arch=x86_64 ;;
  aarch64|arm64) arch=aarch64 ;;
  *) echo "unsupported architecture: $arch" >&2; exit 1 ;;
esac

curl -fsSLO "https://github.com/souk4711/hakoniwa/releases/download/$version/install.sh"
curl -fsSLO "https://github.com/souk4711/hakoniwa/releases/download/$version/SHA256SUMS"

# Verify that the installer and checksums were genuinely produced by this
# repository's release workflow (requires GitHub CLI >= 2.49):
gh attestation verify install.sh --repo souk4711/hakoniwa
gh attestation verify SHA256SUMS --repo souk4711/hakoniwa

sha256sum -c --ignore-missing SHA256SUMS

# Replace this placeholder with the SHA-256 digest for
# hakoniwa-$arch-unknown-linux-gnu.tar.gz from the verified SHA256SUMS file.
HAKONIWA_SHA256=<release-archive-sha256> HAKONIWA_VERSION="$version" bash install.sh
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
