use hakoniwa::*;

fn main() -> Result<()> {
    let mut container = Container::new();
    container.rootfs("/")?;

    let output = container
        .command_from_closure(|| {
            print!("STDOUT");
            eprint!("STDERR");
            1
        })
        .output()?;
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).eq("STDOUT"));
    assert!(String::from_utf8_lossy(&output.stderr).eq("STDERR"));

    Ok(())
}

#[test]
fn test_main() {
    main().unwrap();
}
