use hakoniwa::{Container, Error};

fn main() -> Result<(), Error> {
    let mut container = Container::new();
    container
        .bindmount_ro("/bin", "/bin")
        .bindmount_ro("/lib", "/lib")
        .bindmount_ro("/lib64", "/lib64")
        .bindmount_ro("/usr", "/usr")
        .devfsmount("/dev")
        .tmpfsmount("/tmp");

    let output = container
        .command("/bin/dd")
        .args(["if=/dev/random", "of=/tmp/output.txt", "count=1", "bs=4"])
        .output()?;
    assert!(output.status.success());

    Ok(())
}

#[test]
fn test_main() {
    main().unwrap();
}
