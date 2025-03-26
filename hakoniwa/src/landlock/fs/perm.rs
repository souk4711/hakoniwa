bitflags::bitflags! {
    /// Permission flags.
    #[derive(Hash, Eq, PartialEq, Clone, Copy, Debug)]
    pub struct Perm: u32 {
        const RD   = 1;
        const WR   = 2;
        const EXEC = 4;
    }
}

impl std::fmt::Display for Perm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut str = "".to_string();
        str.push_str(if *self & Self::RD == Self::RD {
            "r"
        } else {
            "-"
        });
        str.push_str(if *self & Self::WR == Self::WR {
            "w"
        } else {
            "-"
        });
        str.push_str(if *self & Self::EXEC == Self::EXEC {
            "x"
        } else {
            "-"
        });
        write!(f, "{}", str)
    }
}

impl std::str::FromStr for Perm {
    type Err = crate::Error;

    fn from_str(mode: &str) -> Result<Self, Self::Err> {
        Ok(match mode.to_lowercase().as_ref() {
            "r--" => Self::RD,
            "rw-" => Self::RD | Self::WR,
            "r-x" => Self::RD | Self::EXEC,
            "rwx" => Self::RD | Self::WR | Self::EXEC,
            "-w-" => Self::WR,
            "-wx" => Self::WR | Self::EXEC,
            "--x" => Self::EXEC,
            perm => {
                let err = format!("unknown permission {}", perm);
                Err(Self::Err::Unexpected(err))?
            }
        })
    }
}
