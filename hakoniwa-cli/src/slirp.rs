use anyhow::Result;
use futures::{SinkExt, StreamExt};
use std::os::fd::RawFd;
use std::{net::SocketAddr, sync::Arc};
use tokio::net::{TcpSocket, TcpStream, UdpSocket};
use tun_rs::AsyncDevice;

pub(crate) fn slirp(tapfd: RawFd) {
    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("failed to create tokio::runtime");
        rt.block_on(async {
            if let Err(e) = slirp_impl(tapfd).await {
                log::error!("slirp: {}", e);
            }
        })
    });
}

async fn slirp_impl(tapfd: RawFd) -> Result<()> {
    // iface
    let iface = netdev::get_default_interface().expect("failed to get default interface");
    let iface = iface.name;

    // TUN device
    let dev = unsafe { AsyncDevice::from_fd(tapfd)? };
    let dev = Arc::new(dev);

    // netstack
    let (stack, runner, udp_socket, tcp_listener) = netstack_smoltcp::StackBuilder::default()
        .enable_tcp(true)
        .enable_udp(true)
        .enable_icmp(true)
        .build()?;
    let (mut stack_sink, mut stack_stream) = stack.split();
    let runner = runner.expect("runner is some");
    let udp_socket = udp_socket.expect("udp_socket is some");
    let tcp_listener = tcp_listener.expect("tcp_listener is some");

    // Futures.
    let mut futs = vec![];
    tokio::spawn(runner);

    // Reads packet from stack and sends to TUN.
    let dev1 = dev.clone();
    futs.push(tokio::spawn(async move {
        while let Some(pkt) = stack_stream.next().await {
            if let Ok(pkt) = pkt
                && let Err(e) = dev1.send(&pkt).await
            {
                log::error!("slirp: failed to send packet to TUN: {}", e);
            }
        }
    }));

    // Reads packet from TUN and sends to stack.
    futs.push(tokio::spawn(async move {
        let mut buf = vec![0; 65536];
        loop {
            if let Ok(len) = dev.recv(&mut buf).await
                && let Err(e) = stack_sink.send(buf[..len].to_vec()).await
            {
                log::error!("slirp: failed to send packet to stack: {}", e);
            }
        }
    }));

    // Extracts TCP connections from stack and sends them to the dispatcher.
    let iface1 = iface.clone();
    futs.push(tokio::spawn(async move {
        handle_inbound_stream(tcp_listener, iface1).await;
    }));

    // Receive and send UDP packets between netstack and NAT manager. The NAT
    // manager would maintain UDP sessions and send them to the dispatcher.
    futs.push(tokio::spawn(async move {
        handle_inbound_datagram(udp_socket, iface).await;
    }));

    // Wait forever.
    futures::future::join_all(futs).await.iter().for_each(|r| {
        if let Err(e) = r {
            log::error!("slirp: {}", e);
        }
    });
    Ok(())
}

async fn handle_inbound_stream(mut tcp_listener: netstack_smoltcp::TcpListener, iface: String) {
    while let Some((mut stream, local, remote)) = tcp_listener.next().await {
        let iface = iface.clone();
        tokio::spawn(async move {
            match new_tcp_stream(remote, &iface).await {
                Ok(mut r) => {
                    if let Err(e) = tokio::io::copy_bidirectional(&mut stream, &mut r).await {
                        let f = "failed to copy tcp stream";
                        log::error!("slirp: {} {} => {}: {}", f, local, remote, e);
                    }
                }
                Err(e) => {
                    let f = "failed to open tcp stream";
                    log::error!("slirp: {} {} => {}: {}", f, local, remote, e);
                }
            }
        });
    }
}

async fn handle_inbound_datagram(udp_socket: netstack_smoltcp::UdpSocket, iface: String) {
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    let (mut read_half, mut write_half) = udp_socket.split();
    tokio::spawn(async move {
        while let Some((data, local, remote)) = rx.recv().await {
            let _ = write_half.send((data, remote, local)).await;
        }
    });
    while let Some((data, local, remote)) = read_half.next().await {
        let tx = tx.clone();
        let iface = iface.clone();
        tokio::spawn(async move {
            match new_udp_packet(remote, &iface).await {
                Ok(sock) => {
                    let _ = sock.send(&data).await;
                    loop {
                        let mut buf = vec![0; 1024];
                        match sock.recv_from(&mut buf).await {
                            Ok((n, _)) => {
                                let _ = tx.send((buf[..n].to_vec(), local, remote));
                            }
                            Err(e) => {
                                log::error!("slirp: udp recv {}: {}", remote, e);
                                break;
                            }
                        }
                    }
                }
                Err(e) => log::error!("slirp: failed to open udp socket {}: {}", remote, e),
            }
        });
    }
}

async fn new_tcp_stream(addr: SocketAddr, iface: &str) -> Result<TcpStream> {
    let s = socket2::Socket::new(socket2::Domain::IPV4, socket2::Type::STREAM, None)?;
    s.bind_device(Some(iface.as_bytes()))?;
    s.set_nonblocking(true)?;
    s.set_keepalive(true)?;
    s.set_tcp_nodelay(true)?;

    let sock = TcpSocket::from_std_stream(s.into());
    Ok(sock.connect(addr).await?)
}

async fn new_udp_packet(addr: SocketAddr, iface: &str) -> Result<UdpSocket> {
    let s = socket2::Socket::new(socket2::Domain::IPV4, socket2::Type::DGRAM, None)?;
    s.bind_device(Some(iface.as_bytes()))?;
    s.set_nonblocking(true)?;

    let sock = UdpSocket::from_std(s.into())?;
    sock.connect(addr).await?;
    Ok(sock)
}
