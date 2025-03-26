use std::str::FromStr;

use hakoniwa::landlock::*;

#[test]
fn test_from_str_r() {
    let perm = FsPerm::from_str("r--").unwrap();
    assert_eq!(perm, FsPerm::RD);

    let perm = FsPerm::from_str("rw-").unwrap();
    assert_eq!(perm, FsPerm::RD | FsPerm::WR);

    let perm = FsPerm::from_str("rwx").unwrap();
    assert_eq!(perm, FsPerm::RD | FsPerm::WR | FsPerm::EXEC);
}

#[test]
fn test_from_str_w() {
    let perm = FsPerm::from_str("-w-").unwrap();
    assert_eq!(perm, FsPerm::WR);

    let perm = FsPerm::from_str("-wx").unwrap();
    assert_eq!(perm, FsPerm::WR | FsPerm::EXEC);

    let perm = FsPerm::from_str("rwx").unwrap();
    assert_eq!(perm, FsPerm::RD | FsPerm::WR | FsPerm::EXEC);
}

#[test]
fn test_from_str_x() {
    let perm = FsPerm::from_str("--x").unwrap();
    assert_eq!(perm, FsPerm::EXEC);

    let perm = FsPerm::from_str("-wx").unwrap();
    assert_eq!(perm, FsPerm::WR | FsPerm::EXEC);

    let perm = FsPerm::from_str("rwx").unwrap();
    assert_eq!(perm, FsPerm::RD | FsPerm::WR | FsPerm::EXEC);
}
