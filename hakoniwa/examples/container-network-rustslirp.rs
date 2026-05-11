#[cfg(feature = "rustslirp")]
fn main() -> Result<(), hakoniwa::Error> {
    use hakoniwa::*;

    let mut container = Container::new();
    container
        .rootfs("/")?
        .unshare(Namespace::Network)
        .network(RustSlirp::default());

    let mut child = container
        .command("/bin/aria2c")
        .args([
            "https://example.com",
            "--async-dns-server=8.8.8.8",
            "--dry-run",
            "--check-certificate=false",
        ])
        .spawn()?;

    let fd = child.rustslirp_tapfd.unwrap();
    let dev = unsafe { tun_rs::SyncDevice::from_fd(fd).unwrap() };
    assert_eq!(dev.name().unwrap(), "tun0");

    std::thread::spawn(move || {
        let mut buf = [0; 65536];
        loop {
            let amount = dev.recv(&mut buf).unwrap();
            let packet = &buf[..amount];
            println!("{:?}", &packet);
        }
    });

    let _output = child.wait_with_output().unwrap();
    // assert!(output.status.success());

    Ok(())
}

#[cfg(not(feature = "rustslirp"))]
fn main() -> Result<(), hakoniwa::Error> {
    Ok(())
}

#[test]
fn test_main() {
    main().unwrap();
}
