use std::fmt;

#[derive(Clone, Debug)]
pub(crate) struct MakeDir {
    pub(crate) target: String,
    pub(crate) mode: u32,
}

#[derive(Clone, Debug)]
pub(crate) struct Symlink {
    pub(crate) original: String,
    pub(crate) link: String,
}

#[derive(Clone, Debug)]
pub(crate) enum Operation {
    MakeDir(MakeDir),
    Symlink(Symlink),
}

impl From<MakeDir> for Operation {
    fn from(val: MakeDir) -> Self {
        Self::MakeDir(val)
    }
}

impl From<Symlink> for Operation {
    fn from(val: Symlink) -> Self {
        Self::Symlink(val)
    }
}

impl fmt::Display for Operation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::MakeDir(dir) => {
                write!(f, "  mkdir: {}", dir.target)
            }
            Self::Symlink(symlink) => {
                write!(f, "symlink: {} -> {}", symlink.link, symlink.original)
            }
        }
    }
}
