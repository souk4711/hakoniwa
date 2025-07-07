bitflags::bitflags! {
    /// FS access flags.
    #[derive(Hash, Eq, PartialEq, Clone, Copy, Debug)]
    pub struct Access: u64 {
        const R = 1;
        const W = 1 << 1;
        const X = 1 << 2;
    }
}

impl std::fmt::Display for Access {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut str = "".to_string();
        for e in [Self::R, Self::W, Self::X] {
            str.push(match *self & e {
                Self::R => 'r',
                Self::W => 'w',
                Self::X => 'x',
                _ => '-',
            });
        }
        write!(f, "{str}")
    }
}

impl std::str::FromStr for Access {
    type Err = crate::Error;

    fn from_str(str: &str) -> Result<Self, Self::Err> {
        let mut access = Self::empty();
        for e in str.chars().collect::<Vec<char>>() {
            match e.to_lowercase().to_string().as_ref() {
                "r" => access |= Self::R,
                "w" => access |= Self::W,
                "x" => access |= Self::X,
                "-" => (),
                chr => {
                    let err = format!("unknown access {chr:?}");
                    Err(Self::Err::LandLockError(err))?
                }
            };
        }
        Ok(access)
    }
}
