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
    /// Constructor.
    pub fn new(arg: u32, op: ArgCmpOp, datum_a: u64, datum_b: u64) -> Self {
        Self {
            arg,
            op,
            datum_a,
            datum_b,
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
