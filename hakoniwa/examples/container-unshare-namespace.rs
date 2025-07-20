use hakoniwa::*;

fn main() -> Result<()> {
    // unshare User, Mount, PID namespaces
    let mut container = Container::new();

    // unshare Cgroup, IPC, Network, UTS namespaces
    container
        .rootfs("/")?
        .unshare(Namespace::Cgroup)
        .unshare(Namespace::Ipc)
        .unshare(Namespace::Network)
        .unshare(Namespace::Uts);

    // require new User namespace
    container.uidmap(0).gidmap(0);
    let user = container
        .command("/bin/id")
        .args(["-u", "-n"])
        .output()
        .unwrap()
        .stdout;
    let group = container
        .command("/bin/id")
        .args(["-g", "-n"])
        .output()
        .unwrap()
        .stdout;
    assert_eq!(String::from_utf8_lossy(&user), "root\n");
    assert_eq!(String::from_utf8_lossy(&group), "root\n");

    // require new UTS namespace
    container.hostname("myhost");
    let hostname = container.command("/bin/hostname").output().unwrap().stdout;
    assert_eq!(String::from_utf8_lossy(&hostname), "myhost\n");

    Ok(())
}

#[test]
fn test_main() {
    main().unwrap();
}
