#[cfg(feature = "landlock")]
fn main() -> Result<(), hakoniwa::Error> {
    use hakoniwa::{landlock::*, *};
    use std::str::FromStr;

    let mut container = Container::new();
    container.rootfs("/")?;

    let mut ruleset = Ruleset::default();
    ruleset.restrict(Resource::FS, CompatMode::Enforce);
    ruleset.allow_path("/bin", FsAccess::from_str("r-x").unwrap());
    ruleset.allow_path("/lib", FsAccess::from_str("r-x").unwrap());
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
