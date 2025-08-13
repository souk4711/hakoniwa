use hakoniwa::*;

fn main() -> Result<()> {
    let mut container = Container::new();
    container
        // .rootfs("/")?   // use `bindmount_ro` & `bindmount_rw` instead of
        .bindmount_ro("/bin", "/bin")
        .bindmount_ro("/lib", "/lib")
        .bindmount_ro("/usr", "/usr")
        .devfsmount("/dev")
        .tmpfsmount("/tmp");

    #[cfg(target_arch = "x86_64")]
    container.bindmount_ro("/lib64", "/lib64");

    let status = container
        .command("/bin/dd")
        .args(["if=/dev/random", "of=/tmp/output.txt", "count=1", "bs=4"])
        .status()?;
    assert!(status.success());

    Ok(())
}

#[test]
fn test_main() {
    main().unwrap();
}
