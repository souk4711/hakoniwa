bitflags::bitflags! {
    /// Access flags.
    #[derive(Hash, Eq, PartialEq, Clone, Copy, Debug)]
    pub struct Access: u32 {
        const R = 1;
        const W = 1 << 1;
        const X = 1 << 2;
    }
}

impl std::fmt::Display for Access {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut str = "".to_string();
        for access in [Self::R, Self::W, Self::X] {
            str.push(if *self & access == access {
                match access {
                    Self::R => 'r',
                    Self::W => 'w',
                    Self::X => 'x',
                    _ => unreachable!(),
                }
            } else {
                '-'
            });
        }
        write!(f, "{}", str)
    }
}

impl std::str::FromStr for Access {
    type Err = crate::Error;

    fn from_str(str: &str) -> Result<Self, Self::Err> {
        let mut mode = Self::empty();
        for access in str.chars().collect::<Vec<char>>() {
            match access.to_lowercase().to_string().as_ref() {
                "r" => mode |= Self::R,
                "w" => mode |= Self::W,
                "x" => mode |= Self::X,
                "-" => (),
                chr => {
                    let err = format!("unknown access {:?}", chr);
                    Err(Self::Err::Unexpected(err))?
                }
            };
        }
        Ok(mode)
    }
}
