use hakoniwa::*;

fn main() -> Result<()> {
    let mut container = Container::new();
    container
        .rootfs("/")?
        .setrlimit(Rlimit::As, 16_000_000, 16_000_000) // 16MB
        .setrlimit(Rlimit::Core, 0, 0) // no core file
        .setrlimit(Rlimit::Fsize, 0, 0) // no output file
        .setrlimit(Rlimit::Nofile, 32, 32); // 32 max fd

    let mut command = container.command("/bin/sleep");
    command.arg("5");
    command.wait_timeout(2); // 2 seconds

    let status = command.status()?;
    assert!(!status.success());
    assert_eq!(status.code, 128 + libc::SIGKILL);

    Ok(())
}

#[test]
fn test_main() {
    main().unwrap();
}
