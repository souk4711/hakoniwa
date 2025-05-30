use std::fmt;

#[derive(Clone, Debug)]
pub(crate) struct WriteFile {
    pub(crate) target: String,
    pub(crate) contents: String,
}

#[derive(Clone, Debug)]
pub(crate) struct MakeDir {
    pub(crate) target: String,
    pub(crate) mode: u32,
}

#[derive(Clone, Debug)]
pub(crate) struct MakeSymlink {
    pub(crate) original: String,
    pub(crate) link: String,
}

#[derive(Clone, Debug)]
pub(crate) enum Operation {
    WriteFile(WriteFile),
    MakeDir(MakeDir),
    MakeSymlink(MakeSymlink),
}

impl From<WriteFile> for Operation {
    fn from(val: WriteFile) -> Self {
        Self::WriteFile(val)
    }
}

impl From<MakeDir> for Operation {
    fn from(val: MakeDir) -> Self {
        Self::MakeDir(val)
    }
}

impl From<MakeSymlink> for Operation {
    fn from(val: MakeSymlink) -> Self {
        Self::MakeSymlink(val)
    }
}

impl fmt::Display for Operation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::WriteFile(file) => {
                write!(f, "  write: {}", file.target)
            }
            Self::MakeDir(dir) => {
                write!(f, "  mkdir: {}", dir.target)
            }
            Self::MakeSymlink(symlink) => {
                write!(f, "symlink: {} -> {}", symlink.link, symlink.original)
            }
        }
    }
}
