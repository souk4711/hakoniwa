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
    let uid = container
        .command("/bin/id")
        .arg("-u")
        .output()
        .unwrap()
        .stdout;
    let gid = container
        .command("/bin/id")
        .arg("-g")
        .output()
        .unwrap()
        .stdout;
    assert_eq!(String::from_utf8_lossy(&uid), "0\n");
    assert_eq!(String::from_utf8_lossy(&gid), "0\n");

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
