/// Network mode.
#[derive(Clone, Debug)]
pub enum Network {
    Pasta(super::Pasta),
}

impl From<super::Pasta> for Network {
    fn from(val: super::Pasta) -> Self {
        Self::Pasta(val)
    }
}
