use hakoniwa::{Container, Error, Namespace, Pasta};

fn main() -> Result<(), Error> {
    let mut container = Container::new();
    container
        .rootfs("/")
        .unshare(Namespace::Network)
        .network(Pasta::default());

    let status = container
        .command("/bin/aria2c")
        .args([
            "https://example.com",
            "--dry-run",
            "-async-dns-server=8.8.8.8",
        ])
        .status()?;
    assert!(status.success());

    Ok(())
}

#[test]
fn test_main() {
    main().unwrap();
}
