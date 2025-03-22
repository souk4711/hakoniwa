use hakoniwa::{Container, Error, Namespace, Pasta};

fn main() -> Result<(), Error> {
    let mut container = Container::new();
    container
        .rootfs("/")
        .unshare(Namespace::Network)
        .network(Pasta::default());

    let status = container
        .command("/bin/wget")
        .args(["https://example.com", "--spider"])
        .status()?;
    assert!(status.success());

    Ok(())
}

#[test]
fn test_main() {
    main().unwrap();
}
