/// Represents a CPU architecture.
#[derive(Hash, Eq, PartialEq, Clone, Copy, Debug)]
pub enum Arch {
    Native,
    X86,
    X8664,
    X32,
    Arm,
    Aarch64,
    Loongarch64,
    M68k,
    Mips,
    Mips64,
    Mips64n32,
    Mipsel,
    Mipsel64,
    Mipsel64n32,
    Ppc,
    Ppc64,
    Ppc64le,
    S390,
    S390x,
    Parisc,
    Parisc64,
    Riscv64,
    Sheb,
    Sh,
}

impl std::str::FromStr for Arch {
    type Err = crate::Error;

    fn from_str(arch: &str) -> Result<Self, Self::Err> {
        Ok(match arch.to_lowercase().as_ref() {
            "x86" => Arch::X86,
            "amd64" | "x86-64" | "x86_64" | "x64" => Arch::X8664,
            "x32" => Arch::X32,
            "arm" => Arch::Arm,
            "arm64" | "aarch64" => Arch::Aarch64,
            "loong64" | "loongarch64" => Arch::Loongarch64,
            "m68k" => Arch::M68k,
            "mips" => Arch::Mips,
            "mips64" => Arch::Mips64,
            "mips64n32" => Arch::Mips64n32,
            "mipsel" => Arch::Mipsel,
            "mipsel64" => Arch::Mipsel64,
            "mipsel64n32" => Arch::Mipsel64n32,
            "ppc" => Arch::Ppc,
            "ppc64" => Arch::Ppc64,
            "ppc64le" => Arch::Ppc64le,
            "s390" => Arch::S390,
            "s390x" => Arch::S390x,
            "parisc" => Arch::Parisc,
            "parisc64" => Arch::Parisc64,
            "riscv64" => Arch::Riscv64,
            "sheb" => Arch::Sheb,
            "sh" => Arch::Sh,
            arch => {
                let err = format!("unsupported architectures {arch}");
                Err(Self::Err::Unexpected(err))?
            }
        })
    }
}
