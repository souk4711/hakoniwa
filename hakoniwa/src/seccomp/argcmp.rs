/// Represents a comparison operator which can be used in an ArgCmp.
#[derive(Hash, Eq, PartialEq, Clone, Copy)]
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
#[derive(Clone)]
pub struct ArgCmp {
    pub(crate) arg: u32,
    pub(crate) op: ArgCmpOp,
    pub(crate) datum: u64,
    pub(crate) datum_two: Option<u64>,
}

impl ArgCmp {
    /// Constructor.
    pub fn new(arg: u32, op: ArgCmpOp, datum: u64, datum_two: Option<u64>) -> Self {
        Self {
            arg,
            op,
            datum,
            datum_two,
        }
    }
}
