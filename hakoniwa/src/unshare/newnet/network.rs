/// Network mode.
#[derive(Clone, Debug)]
pub enum Network {
    Pasta(super::Pasta),
    #[cfg(feature = "rustslirp")]
    RustSlirp(super::rustslirp::RustSlirp),
}

impl From<super::Pasta> for Network {
    fn from(val: super::Pasta) -> Self {
        Self::Pasta(val)
    }
}

#[cfg(feature = "rustslirp")]
impl From<super::rustslirp::RustSlirp> for Network {
    fn from(val: super::rustslirp::RustSlirp) -> Self {
        Self::RustSlirp(val)
    }
}
