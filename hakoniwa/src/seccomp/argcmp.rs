use std::fmt;

/// Represents a comparison operator which can be used in an ArgCmp.
#[derive(Hash, Eq, PartialEq, Clone, Copy, Debug)]
pub enum ArgCmpOp {
    Ne,
    Lt,
    Le,
    Eq,
    Gt,
    Ge,
    MaskedEq,
}

/// Represents an argument comparison rule.
#[derive(Hash, Eq, PartialEq, Clone, Copy, Debug)]
pub struct ArgCmp {
    pub(crate) arg: u32,
    pub(crate) op: ArgCmpOp,
    pub(crate) datum_a: u64,
    pub(crate) datum_b: u64,
}

impl ArgCmp {
    /// Creates and returns a new condition to attach to a filter rule.
    ///
    /// You can use the [scmp_argcmp!] macro instead of this to create ArgCmp
    /// in a more elegant way.
    ///
    /// [scmp_argcmp!]: crate::scmp_argcmp
    pub fn new(arg: u32, op: ArgCmpOp, datum_a: u64, datum_b: u64) -> Self {
        Self {
            arg,
            op,
            datum_a,
            datum_b,
        }
    }
}

impl fmt::Display for ArgCmp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.op {
            ArgCmpOp::Ne => write!(f, "${} != {}", self.arg, self.datum_a),
            ArgCmpOp::Lt => write!(f, "${} < {}", self.arg, self.datum_a),
            ArgCmpOp::Le => write!(f, "${} <= {}", self.arg, self.datum_a),
            ArgCmpOp::Eq => write!(f, "${} == {}", self.arg, self.datum_a),
            ArgCmpOp::Gt => write!(f, "${} > {}", self.arg, self.datum_a),
            ArgCmpOp::Ge => write!(f, "${} >= {}", self.arg, self.datum_a),
            ArgCmpOp::MaskedEq => write!(f, "${} & {} = {}", self.arg, self.datum_a, self.datum_b),
        }
    }
}

#[rustfmt::skip]
#[doc(hidden)]
#[macro_export]
macro_rules! scmp_argcmp_arg {
    (arg0) => { 0 };
    (arg1) => { 1 };
    (arg2) => { 2 };
    (arg3) => { 3 };
    (arg4) => { 4 };
    (arg5) => { 5 };
}

/// E.g.
///
/// ```
/// use hakoniwa::{scmp_argcmp, seccomp::*};
///
/// assert_eq!(
///     scmp_argcmp!(arg0 != 123),
///     ArgCmp::new(0, ArgCmpOp::Ne, 123, 0),
/// );
/// assert_eq!(
///     scmp_argcmp!(arg1 < 123),
///     ArgCmp::new(1, ArgCmpOp::Lt, 123, 0),
/// );
/// assert_eq!(
///     scmp_argcmp!(arg2 <= 123),
///     ArgCmp::new(2, ArgCmpOp::Le, 123, 0),
/// );
/// assert_eq!(
///     scmp_argcmp!(arg3 == 123),
///     ArgCmp::new(3, ArgCmpOp::Eq, 123, 0),
/// );
/// assert_eq!(
///     scmp_argcmp!(arg4 > 123),
///     ArgCmp::new(4, ArgCmpOp::Gt, 123, 0),
/// );
/// assert_eq!(
///     scmp_argcmp!(arg5 >= 123),
///     ArgCmp::new(5, ArgCmpOp::Ge, 123, 0),
/// );
/// assert_eq!(
///     scmp_argcmp!(arg0 & 0b0010 == 123),
///     ArgCmp::new(0, ArgCmpOp::MaskedEq, 0b0010, 123),
/// );
/// ```
#[rustfmt::skip]
#[macro_export]
macro_rules! scmp_argcmp {
    ($arg:ident != $datum:expr) => {
        ArgCmp::new($crate::scmp_argcmp_arg!($arg), $crate::seccomp::ArgCmpOp::Ne, $datum, 0)
    };

    ($arg:ident < $datum:expr) => {
        ArgCmp::new($crate::scmp_argcmp_arg!($arg), $crate::seccomp::ArgCmpOp::Lt, $datum, 0)
    };

    ($arg:ident <= $datum:expr) => {
        ArgCmp::new($crate::scmp_argcmp_arg!($arg), $crate::seccomp::ArgCmpOp::Le, $datum, 0)
    };

    ($arg:ident == $datum:expr) => {
        ArgCmp::new($crate::scmp_argcmp_arg!($arg), $crate::seccomp::ArgCmpOp::Eq, $datum, 0)
    };

    ($arg:ident > $datum:expr) => {
        ArgCmp::new($crate::scmp_argcmp_arg!($arg), $crate::seccomp::ArgCmpOp::Gt, $datum, 0)
    };

    ($arg:ident >= $datum:expr) => {
        ArgCmp::new($crate::scmp_argcmp_arg!($arg), $crate::seccomp::ArgCmpOp::Ge, $datum, 0)
    };

    ($arg:ident & $mask:literal == $datum:expr) => {
        ArgCmp::new($crate::scmp_argcmp_arg!($arg), $crate::seccomp::ArgCmpOp::MaskedEq, $mask, $datum)
    };
}
