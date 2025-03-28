#[cfg(feature = "landlock")]
fn main() -> Result<(), hakoniwa::Error> {
    use hakoniwa::{landlock::*, *};

    let mut container = Container::new();
    container.rootfs("/");

    let mut ruleset = Ruleset::default();
    ruleset.add_fs_rule("/bin", FsPerm::RD | FsPerm::EXEC);
    ruleset.add_fs_rule("/lib", FsPerm::RD | FsPerm::EXEC);
    container.landlock_ruleset(ruleset);

    let output = container.command("/bin/cat").arg("/etc/hosts").output()?;
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("Permission denied"));

    Ok(())
}

#[cfg(not(feature = "landlock"))]
fn main() -> Result<(), hakoniwa::Error> {
    Ok(())
}

#[test]
fn test_main() {
    main().unwrap();
}
