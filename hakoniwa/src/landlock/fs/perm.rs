bitflags::bitflags! {
    /// Permission flags.
    #[derive(Hash, Eq, PartialEq, Clone, Copy, Debug)]
    pub struct Perm: u32 {
        const RD   = 1;
        const WR   = 2;
        const EXEC = 4;
    }
}

impl std::str::FromStr for Perm {
    type Err = crate::Error;

    fn from_str(mode: &str) -> Result<Self, Self::Err> {
        Ok(match mode.to_lowercase().as_ref() {
            "r--" => Perm::RD,
            "rw-" => Perm::RD | Perm::WR,
            "r-x" => Perm::RD | Perm::EXEC,
            "rwx" => Perm::RD | Perm::WR | Perm::EXEC,
            "-w-" => Perm::WR,
            "-wx" => Perm::WR | Perm::EXEC,
            "--x" => Perm::EXEC,
            perm => {
                let err = format!("unknown permission {}", perm);
                Err(Self::Err::Unexpected(err))?
            }
        })
    }
}
