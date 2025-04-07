use std::fmt;

#[derive(Clone, Debug)]
pub(crate) struct Symlink {
    pub(crate) original: String,
    pub(crate) link: String,
}

#[derive(Clone, Debug)]
pub(crate) enum Operation {
    Symlink(Symlink),
}

impl From<Symlink> for Operation {
    fn from(val: Symlink) -> Self {
        Self::Symlink(val)
    }
}

impl fmt::Display for Operation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Symlink(symlink) => {
                write!(f, "symlink: {} -> {}", symlink.link, symlink.original)
            }
        }
    }
}
