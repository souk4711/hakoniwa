use nix::sched::{CloneFlags, setns};
use nix::sys::socket::{AddressFamily, SockFlag, SockType, socket};
use nix::unistd::{self, ForkResult, Pid};
use sendfd::{RecvWithFd, SendWithFd};
use std::fs::File;
use std::os::fd::{AsRawFd, RawFd};
use std::os::unix::net::UnixDatagram;
use tun_rs::DeviceBuilder;

use super::SetupStatus;
use crate::error::*;

nix::ioctl_readwrite_bad!(get_iface_flags, libc::SIOCGIFFLAGS, libc::ifreq);
nix::ioctl_readwrite_bad!(set_iface_flags, libc::SIOCSIFFLAGS, libc::ifreq);

/// User-mode networking for unprivileged network namespaces.
///
/// Creates a TUN/TAP device in the new NETWORK namespace, and returns the tapfd to
/// the caller. This allows the caller to manage packets / do filtering explicitly.
#[derive(Clone, Debug)]
pub struct RustSlirp {
    mtu: u16,
}

impl RustSlirp {
    /// Sets the device MTU (Maximum Transmission Unit).
    pub fn mtu(&mut self, mtu: u16) -> &mut Self {
        self.mtu = mtu;
        self
    }
}

impl Default for RustSlirp {
    fn default() -> Self {
        Self { mtu: 1500 }
    }
}

impl RustSlirp {
    pub(crate) fn mainp_setup(rustslirp: &Self, child: Pid) -> Result<SetupStatus> {
        // Create a pair of sockets to transfer tapfd.
        let (sock_a, sock_z) = UnixDatagram::pair().map_err(ProcessErrorKind::StdIoError)?;

        match unsafe { unistd::fork() } {
            Ok(ForkResult::Parent { .. }) => {
                let fd = Self::mainp_setup_parent(sock_a).map_err(|err| {
                    let errmsg = format!("{err}");
                    ProcessErrorKind::SetupNetworkFailed(errmsg)
                })?;
                Ok(SetupStatus::RustSlirpTapFd(fd))
            }
            Ok(ForkResult::Child) => {
                Self::mainp_setup_child(sock_z, rustslirp, child).map_err(|err| {
                    let errmsg = format!("rustslirp: {err}");
                    unsafe {
                        libc::write(
                            libc::STDERR_FILENO,
                            errmsg.as_ptr().cast(),
                            errmsg.len() as libc::size_t,
                        );
                        libc::_exit(1)
                    }
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
        let mut buf = [];
        let mut fds = [0 as RawFd];
        sock.recv_with_fd(&mut buf, &mut fds)
            .map_err(ProcessErrorKind::StdIoError)?;
        Ok(fds[0])
    }

    // [slirp4netns#child]: https://github.com/rootless-containers/slirp4netns/blob/944fa94090e1fd1312232cbc0e6b43585553d824/main.c#L236
    fn mainp_setup_child(sock: UnixDatagram, rustslirp: &RustSlirp, child: Pid) -> Result<()> {
        // Change into target's NETWORK namespace.
        let fd1 =
            File::open(format!("/proc/{}/ns/net", child)).map_err(ProcessErrorKind::StdIoError)?;
        let fd2 =
            File::open(format!("/proc/{}/ns/user", child)).map_err(ProcessErrorKind::StdIoError)?;
        setns(fd2, CloneFlags::CLONE_NEWUSER).map_err(ProcessErrorKind::NixError)?;
        setns(fd1, CloneFlags::CLONE_NEWNET).map_err(ProcessErrorKind::NixError)?;

        // Bring up loopback Interface.
        Self::bring_up_loopback_interface()?;

        // Create a TUN/TAP device.
        let dev = DeviceBuilder::new()
            .mtu(rustslirp.mtu)
            .ipv4("10.0.0.1", 24, None)
            .build_sync()
            .map_err(ProcessErrorKind::StdIoError)?;

        // Add a default route.
        let destination = "0.0.0.0".parse().expect("failed to parse ip address");
        let route = route_manager::Route::new(destination, 0).with_if_index(2);
        let mut mgr = route_manager::RouteManager::new().map_err(ProcessErrorKind::StdIoError)?;
        mgr.add(&route).map_err(ProcessErrorKind::StdIoError)?;

        // Sent back tapfd.
        sock.send_with_fd(&[], &[dev.as_raw_fd()])
            .map_err(ProcessErrorKind::StdIoError)?;
        Ok(())
    }

    fn bring_up_loopback_interface() -> Result<()> {
        let fd = socket(AddressFamily::Inet, SockType::Datagram, SockFlag::empty(), None).map_err(ProcessErrorKind::NixError)?;

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
}
