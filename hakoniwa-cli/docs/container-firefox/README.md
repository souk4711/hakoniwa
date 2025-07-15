# Contianer App - Firefox

## Launch With Command Line Arguments

### 1. Create Rootfs From Container Image

```sh
mkdir -p ~/hakoniwa/images/archlinux
podman export $(podman create archlinux) | tar -C ~/hakoniwa/images/archlinux -xf -
```

### 2. Chroot Into Rootfs

```sh
hakoniwa run -v \
  --unshare-all \
  --rootdir ~/hakoniwa/images/archlinux:rw \
  -b /dev -b /sys \
  --network=pasta -b /etc/resolv.conf \
  --userns=auto \
  --hostname hakoniwa \
  -- /usr/bin/bash
```

### 3. Install Firefox

```console
[root@hakoniwa /]# sed -i 's/#Color/Color/g' /etc/pacman.conf
[root@hakoniwa /]# sed -i 's/NoProgressBar/#NoProgressBar/g' /etc/pacman.conf

[root@hakoniwa /]# pacman-key --init && pacman-key --populate && pacman -Syu --noconfirm
==> Generating pacman master key. This may take some time.
==> Updating trust database...
==> Generating pacman master key. This may take some time.
==> Updating trust database...
:: Synchronizing package databases...
:: Starting full system upgrade...
:: Retrieving packages...
:: Processing package changes...
:: Running post-transaction hooks...
...

[root@hakoniwa /]# pacman -S --noconfirm noto-fonts-cjk firefox
resolving dependencies...
looking for conflicting packages...
:: Retrieving packages...
:: Processing package changes...
:: Running post-transaction hooks...
...

[root@hakoniwa /]# exit
```

### 4. Launch Firefox

```sh
mkdir -p ~/hakoniwa/apps/archlinux-firefox
hakoniwa run -v \
  --unshare-all \
  --rootfs ~/hakoniwa/images/archlinux --devfs /dev --tmpfs /tmp --tmpfs /run --tmpfs /home \
  -b /dev/dri -b /dev/snd -b /sys \
  -b /tmp/.X11-unix -e DISPLAY -b "$XAUTHORITY" -e XAUTHORITY \
  -b /run/dbus/system_bus_socket -b "$XDG_RUNTIME_DIR/bus" -e DBUS_SESSION_BUS_ADDRESS \
  --network=pasta -b /etc/resolv.conf \
  -B "$HOME/hakoniwa/apps/archlinux-firefox":"$HOME" -e HOME \
  -B "$HOME/Downloads" \
  -- /usr/bin/firefox
```
