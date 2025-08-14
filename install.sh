#!/usr/bin/env bash


set -euo pipefail


command_exists() {
  type "${1}" > /dev/null 2>&1
}

echo_info() {
  echo -e "\e[1;34mINFO: $* \e[m"
}

echo_warn() {
  echo -e "\e[1;33mWARN: $* \e[m"
}

echo_error() {
  echo -e "\e[1;31mERROR: $* \e[m"
  exit 1
}

get_arch() {
  case $(uname -m) in
    amd64|x86_64) echo "x86_64";;
    aarch64|arm64) echo "aarch64";;
    *) "NOT_SUPPORTED";;
  esac
}

# shellcheck disable=SC1091
get_distro() {
  if [ -f /etc/os-release ]; then
    (
      . /etc/os-release
      if [ "${ID_LIKE-}" ]; then
        for id_like in $ID_LIKE; do
          case "$id_like" in arch | debian | fedora)
            echo "$id_like"
            return
            ;;
          esac
        done
      fi
      echo "$ID"
      return
    )
  fi
}


install_deps() {
  echo ""
  echo_info "Installing dependencies..."

  case $DISTRO in
    arch)
      echo "pacman -S --noconfirm libseccomp passt shadow"
      sudo pacman -S --noconfirm libseccomp passt shadow
      ;;
    debian)
      echo "apt install -y libseccomp-dev passt uidmap"
      sudo apt install -y libseccomp-dev passt uidmap
      ;;
    fedora)
      echo "dnf install -y libseccomp-devel passt shadow-utils"
      sudo dnf install -y libseccomp-devel passt shadow-utils
      ;;
    *)
      echo_warn "Distro $DISTRO is not supported, please manually install dependencies."
      ;;
  esac
}

install_hakoniwa() {
  echo ""
  echo_info "Installing hakoniwa..."

  filename="hakoniwa-$ARCH-unknown-linux-gnu.tar.gz"
  url="https://github.com/souk4711/hakoniwa/releases/latest/download/$filename"

  echo "curl -L --progress-bar -o $CACHE_DIR/$filename $url"
  curl -L --progress-bar -o "$CACHE_DIR/$filename" "$url"

  echo "tar -xzf $CACHE_DIR/$filename -C $CACHE_DIR"
  tar -xzf "$CACHE_DIR/$filename" -C "$CACHE_DIR"

  echo "cp $CACHE_DIR/hakoniwa /usr/bin/hakoniwa"
  sudo cp "$CACHE_DIR/hakoniwa" /usr/bin/hakoniwa
}

configure_selinux() {
  echo ""
  echo_info "Configuring SELinux..."

  if ! command_exists "getenforce"; then
    echo "module SELinux not found. SKIPPING."
    return
  fi

  case $DISTRO in
    fedora)
      echo "dnf install -y container-selinux"
      sudo dnf install -y container-selinux
      echo "chcon -u system_u -t container_runtime_exec_t /usr/bin/hakoniwa"
      sudo chcon -u system_u -t container_runtime_exec_t /usr/bin/hakoniwa
      ;;
    *)
      echo_warn "Distro $DISTRO is not supported, please manually configure SELinux."
      ;;
  esac
}

configure_apparmor() {
  echo ""
  echo_info "Configuring AppArmor..."

  if ! command_exists "apparmor_status"; then
    echo "module AppArmor not found. SKIPPING."
    return
  fi

  echo "cat <<EOF | tee /etc/apparmor.d/hakoniwa >/dev/null"
  echo "......"
  echo "EOF"
  cat <<EOF | sudo tee /etc/apparmor.d/hakoniwa >/dev/null
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

  if apparmor_status --enabled; then
    echo "systemctl reload apparmor.service"
    sudo systemctl reload apparmor.service
  fi
}


main() {
  if command_exists "/usr/bin/hakoniwa"; then
    echo_warn "/usr/bin/hakoniwa already exists."
    exit 0
  fi

  ARCH=${ARCH:-$(get_arch)}
  DISTRO=${DISTRO:-$(get_distro)}
  CACHE_DIR=$(mktemp -d)

  echo_info "variables:"
  echo "ARCH=$ARCH"
  echo "DISTRO=$DISTRO"
  echo "CACHE_DIR=$CACHE_DIR"

  case $ARCH in
    x86_64 | aarch64) ;;
    *) echo_error "Unsupported architecture: $ARCH";;
  esac
  case $DISTRO in
    arch | debian | fedora) ;;
    *) echo_error "Unsupported distro: $DISTRO";;
  esac

  install_deps
  install_hakoniwa
  configure_selinux
  configure_apparmor
}
main "$@"
