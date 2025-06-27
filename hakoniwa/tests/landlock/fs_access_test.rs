use std::str::FromStr;

use hakoniwa::landlock::*;

#[test]
fn test_format_r() {
    let mode = FsAccess::R;
    assert_eq!(format!("{mode}"), "r--");

    let mode = FsAccess::R | FsAccess::W;
    assert_eq!(format!("{mode}"), "rw-");

    let mode = FsAccess::R | FsAccess::W | FsAccess::X;
    assert_eq!(format!("{mode}"), "rwx");
}

#[test]
fn test_format_w() {
    let mode = FsAccess::W;
    assert_eq!(format!("{mode}"), "-w-");

    let mode = FsAccess::W | FsAccess::X;
    assert_eq!(format!("{mode}"), "-wx");
}

#[test]
fn test_format_x() {
    let mode = FsAccess::X;
    assert_eq!(format!("{mode}"), "--x");
}

#[test]
fn test_from_str_r() {
    let mode = FsAccess::from_str("r--").unwrap();
    assert_eq!(mode, FsAccess::R);

    let mode = FsAccess::from_str("rw-").unwrap();
    assert_eq!(mode, FsAccess::R | FsAccess::W);

    let mode = FsAccess::from_str("rwx").unwrap();
    assert_eq!(mode, FsAccess::R | FsAccess::W | FsAccess::X);

    let mode = FsAccess::from_str("r").unwrap();
    assert_eq!(mode, FsAccess::R);

    let mode = FsAccess::from_str("rw").unwrap();
    assert_eq!(mode, FsAccess::R | FsAccess::W);
}

#[test]
fn test_from_str_w() {
    let mode = FsAccess::from_str("-w-").unwrap();
    assert_eq!(mode, FsAccess::W);

    let mode = FsAccess::from_str("-wx").unwrap();
    assert_eq!(mode, FsAccess::W | FsAccess::X);

    let mode = FsAccess::from_str("rwx").unwrap();
    assert_eq!(mode, FsAccess::R | FsAccess::W | FsAccess::X);

    let mode = FsAccess::from_str("w").unwrap();
    assert_eq!(mode, FsAccess::W);

    let mode = FsAccess::from_str("wx").unwrap();
    assert_eq!(mode, FsAccess::W | FsAccess::X);
}

#[test]
fn test_from_str_x() {
    let mode = FsAccess::from_str("--x").unwrap();
    assert_eq!(mode, FsAccess::X);

    let mode = FsAccess::from_str("-wx").unwrap();
    assert_eq!(mode, FsAccess::W | FsAccess::X);

    let mode = FsAccess::from_str("rwx").unwrap();
    assert_eq!(mode, FsAccess::R | FsAccess::W | FsAccess::X);

    let mode = FsAccess::from_str("x").unwrap();
    assert_eq!(mode, FsAccess::X);

    let mode = FsAccess::from_str("wx").unwrap();
    assert_eq!(mode, FsAccess::W | FsAccess::X);
}
