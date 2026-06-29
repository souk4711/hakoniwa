use nix::sched::{CloneFlags, setns};
use nix::sys::socket::{AddressFamily, SockFlag, SockType, socket};
use nix::unistd::{self, ForkResult, Pid};
use sendfd::{RecvWithFd, SendWithFd};
use std::fs::File;
use std::net::Ipv4Addr;
use std::os::fd::{AsRawFd, RawFd};
use std::os::unix::net::UnixDatagram;
use tun_rs::{DeviceBuilder, Layer, SyncDevice};

use super::SetupStatus;
use crate::error::*;

nix::ioctl_readwrite_bad!(get_iface_flags, libc::SIOCGIFFLAGS, libc::ifreq);
nix::ioctl_readwrite_bad!(set_iface_flags, libc::SIOCSIFFLAGS, libc::ifreq);

/// RustSlirp mode.
#[allow(clippy::upper_case_acronyms)]
#[derive(Clone, Debug)]
pub enum RustSlirpMode {
    TUN,
    TAP,
}

/// Configuration for the default gateway within the network namespace.
#[derive(Clone, Debug)]
pub enum RustSlirpGateway {
    None,
    IfaceOnly,
    IfaceWithAddr(Ipv4Addr),
}

/// User-mode networking for unprivileged network namespaces.
///
/// Creates a TUN/TAP device in the new NETWORK namespace, and returns the tapfd to
/// the caller. This allows the caller to manage packets / do filtering explicitly.
#[derive(Clone, Debug)]
pub struct RustSlirp {
    mode: RustSlirpMode,
    address: Ipv4Addr,
    netmask: Ipv4Addr,
    destination: Option<Ipv4Addr>,
    gateway: RustSlirpGateway,
    mtu: u16,
}

impl RustSlirp {
    /// Creating a TUN(L3) or TAP (L2) interface.
    pub fn mode(&mut self, mode: RustSlirpMode) -> &mut Self {
        self.mode = mode;
        self
    }

    /// Sets the address.
    pub fn address(&mut self, address: Ipv4Addr) -> &mut Self {
        self.address = address;
        self
    }

    /// Sets the netmask.
    pub fn netmask(&mut self, netmask: Ipv4Addr) -> &mut Self {
        self.netmask = netmask;
        self
    }

    /// Sets the destination.
    pub fn destination(&mut self, destination: Ipv4Addr) -> &mut Self {
        self.destination = Some(destination);
        self
    }

    /// Sets the device MTU (Maximum Transmission Unit).
    pub fn mtu(&mut self, mtu: u16) -> &mut Self {
        self.mtu = mtu;
        self
    }

    /// Sets the default gateway.
    pub fn gateway(&mut self, gateway: RustSlirpGateway) -> &mut Self {
        self.gateway = gateway;
        self
    }
}

impl Default for RustSlirp {
    fn default() -> Self {
        Self {
            mode: RustSlirpMode::TUN,
            address: Ipv4Addr::new(10, 0, 0, 1),
            netmask: Ipv4Addr::new(255, 255, 255, 0),
            destination: None,
            gateway: RustSlirpGateway::IfaceOnly,
            mtu: 1500,
        }
    }
}

impl RustSlirp {
    pub(crate) fn mainp_setup(rustslirp: &Self, target: Pid) -> Result<SetupStatus> {
        // Create a pair of sockets to transfer tapfd.
        let (sock_a, sock_z) = UnixDatagram::pair().map_err(ProcessErrorKind::StdIoError)?;

        match unsafe { unistd::fork() } {
            Ok(ForkResult::Parent { .. }) => {
                drop(sock_z);
                let fd = Self::mainp_setup_parent(sock_a).map_err(|err| {
                    let errmsg = format!("{err}");
                    ProcessErrorKind::SetupNetworkFailed(errmsg)
                })?;
                Ok(SetupStatus::RustSlirpTapFd(fd))
            }
            Ok(ForkResult::Child) => {
                drop(sock_a);
                Self::mainp_setup_child(&sock_z, rustslirp, target).map_err(|err| {
                    let errmsg = format!("rustslirp: {err}");
                    _ = sock_z.send_with_fd(errmsg.as_bytes(), &[]);
                    unsafe { libc::_exit(1) }
                });
                unsafe { libc::_exit(0) }
            }
            Err(err) => {
                let errmsg = format!("{err}");
                Err(ProcessErrorKind::SetupNetworkFailed(errmsg))?
            }
        }
    }

    // [slirp4netns#main]: https://github.com/rootless-containers/slirp4netns/blob/944fa94090e1fd1312232cbc0e6b43585553d824/main.c#L1130
    fn mainp_setup_parent(sock: UnixDatagram) -> Result<RawFd> {
        // Recv tapfd.
        let mut buf = [0u8; 1024];
        let mut fds = [0 as RawFd];
        let (bufsize, _) = sock
            .recv_with_fd(&mut buf, &mut fds)
            .map_err(ProcessErrorKind::StdIoError)?;
        if bufsize != 0 {
            let errmsg = String::from_utf8_lossy(&buf[..bufsize]);
            Err(Error::UnError(errmsg.to_string()))
        } else {
            Ok(fds[0])
        }
    }

    // [slirp4netns#child]: https://github.com/rootless-containers/slirp4netns/blob/944fa94090e1fd1312232cbc0e6b43585553d824/main.c#L236
    fn mainp_setup_child(sock: &UnixDatagram, rustslirp: &RustSlirp, target: Pid) -> Result<()> {
        // Change into target's NETWORK namespace.
        let fd1 =
            File::open(format!("/proc/{}/ns/net", target)).map_err(ProcessErrorKind::StdIoError)?;
        let fd2 = File::open(format!("/proc/{}/ns/user", target))
            .map_err(ProcessErrorKind::StdIoError)?;
        setns(fd2, CloneFlags::CLONE_NEWUSER).map_err(ProcessErrorKind::NixError)?;
        setns(fd1, CloneFlags::CLONE_NEWNET).map_err(ProcessErrorKind::NixError)?;

        // Bring up loopback Interface.
        Self::bring_up_loopback_interface()?;

        // Create a TUN/TAP device.
        let dev = Self::create_tun_device(rustslirp)?;

        // Add the default route, if configured.
        if !matches!(rustslirp.gateway, RustSlirpGateway::None) {
            let r = route_manager::Route::new(
                "0.0.0.0".parse().expect("failed to parse ip address"),
                0,
            );
            let route = match &rustslirp.gateway {
                RustSlirpGateway::IfaceOnly => r.with_if_index(2),
                RustSlirpGateway::IfaceWithAddr(ip) => {
                    r.with_gateway(std::net::IpAddr::V4(*ip)).with_if_index(2)
                }
                RustSlirpGateway::None => unreachable!(),
            };
            let mut mgr =
                route_manager::RouteManager::new().map_err(ProcessErrorKind::StdIoError)?;
            mgr.add(&route).map_err(ProcessErrorKind::StdIoError)?;
        }

        // Sent back tapfd.
        _ = sock.send_with_fd(b"", &[dev.as_raw_fd()]);
        Ok(())
    }

    fn bring_up_loopback_interface() -> Result<()> {
        let fd = socket(
            AddressFamily::Inet,
            SockType::Datagram,
            SockFlag::empty(),
            None,
        )
        .map_err(ProcessErrorKind::NixError)?;

        let if_name = "lo";
        let mut ifr: libc::ifreq = unsafe { std::mem::zeroed() };
        for (i, byte) in if_name.as_bytes().iter().enumerate() {
            ifr.ifr_name[i] = *byte as libc::c_char;
        }

        unsafe {
            get_iface_flags(fd.as_raw_fd(), &mut ifr).map_err(ProcessErrorKind::NixError)?;

            ifr.ifr_ifru.ifru_flags |= (libc::IFF_UP | libc::IFF_RUNNING) as libc::c_short;
            set_iface_flags(fd.as_raw_fd(), &mut ifr).map_err(ProcessErrorKind::NixError)?;
        }
        Ok(())
    }

    fn create_tun_device(rustslirp: &RustSlirp) -> Result<SyncDevice> {
        let mut builder = DeviceBuilder::new();

        match rustslirp.mode {
            RustSlirpMode::TUN => builder = builder.layer(Layer::L3),
            RustSlirpMode::TAP => builder = builder.layer(Layer::L2),
        };

        let dev = builder
            .mtu(rustslirp.mtu)
            .ipv4(rustslirp.address, rustslirp.netmask, rustslirp.destination)
            .build_sync()
            .map_err(ProcessErrorKind::StdIoError)?;
        Ok(dev)
    }
}
