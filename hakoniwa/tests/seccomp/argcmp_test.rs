use hakoniwa::{scmp_argcmp, seccomp::*};

#[test]
fn test_scmp_argcmp_ne() {
    assert_eq!(
        scmp_argcmp!(arg0 != 123),
        ArgCmp::new(0, ArgCmpOp::Ne, 123, 0),
    );
}

#[test]
fn test_scmp_argcmp_lt() {
    assert_eq!(
        scmp_argcmp!(arg1 < 123),
        ArgCmp::new(1, ArgCmpOp::Lt, 123, 0),
    );
}

#[test]
fn test_scmp_argcmp_le() {
    assert_eq!(
        scmp_argcmp!(arg2 <= 123),
        ArgCmp::new(2, ArgCmpOp::Le, 123, 0),
    );
}

#[test]
fn test_scmp_argcmp_eq() {
    assert_eq!(
        scmp_argcmp!(arg3 == 123),
        ArgCmp::new(3, ArgCmpOp::Eq, 123, 0),
    );
}

#[test]
fn test_scmp_argcmp_gt() {
    assert_eq!(
        scmp_argcmp!(arg4 > 123),
        ArgCmp::new(4, ArgCmpOp::Gt, 123, 0),
    );
}

#[test]
fn test_scmp_argcmp_ge() {
    assert_eq!(
        scmp_argcmp!(arg5 >= 123),
        ArgCmp::new(5, ArgCmpOp::Ge, 123, 0),
    );
}

#[test]
fn test_scmp_argcmp_masked_eq() {
    assert_eq!(
        scmp_argcmp!(arg0 & 0b0010 == 123),
        ArgCmp::new(0, ArgCmpOp::MaskedEq, 0b0010, 123),
    );
}
