use hakoniwa::{Container, Error, Namespace};
use nix::unistd::{Gid, Uid};

fn main() -> Result<(), Error> {
    // unshare User, Mount, PID
    let mut container = Container::new();

    // unshare Cgroup, IPC, Network, UTS
    container
        .unshare(Namespace::Cgroup)
        .unshare(Namespace::Ipc)
        .unshare(Namespace::Network)
        .unshare(Namespace::Uts);

    // require new User namespace
    container.uidmap(0).gidmap(0);
    let uid = container.command("id").arg("-u").output().unwrap().stdout;
    let gid = container.command("id").arg("-g").output().unwrap().stdout;
    assert_eq!(
        String::from_utf8_lossy(&uid),
        Uid::current().as_raw().to_string()
    );
    assert_eq!(
        String::from_utf8_lossy(&gid),
        Gid::current().as_raw().to_string()
    );

    // require new UTS namespace
    container.hostname("myhost");
    let hostname = container.command("hostname").output().unwrap().stdout;
    assert_eq!(String::from_utf8_lossy(&hostname), "myhost\n");

    Ok(())
}

#[test]
fn test_main() {
    main().unwrap();
}
