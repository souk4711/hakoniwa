use hakoniwa::*;

fn main() -> Result<()> {
    let mut container = Container::new();
    container.rootfs("/")?;

    let mut command = unsafe {
        container.command_from_closure(|| {
            _ = nix::unistd::write(std::io::stdout(), b"STDOUT");
            _ = nix::unistd::write(std::io::stderr(), b"STDERR");
            1
        })
    };

    let output = command.output()?;
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).eq("STDOUT"));
    assert!(String::from_utf8_lossy(&output.stderr).eq("STDERR"));

    Ok(())
}

#[test]
fn test_main() {
    main().unwrap();
}
