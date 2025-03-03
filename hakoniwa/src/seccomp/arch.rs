/// Represents a CPU architecture.
#[derive(Hash, Eq, PartialEq, Clone, Copy)]
pub enum Arch {
    X8664,
    X86,
    X32,
    Aarch64,
    Arm,
    Mips64n32,
    Mips64,
    Mips,
    Mipsel64n32,
    Mipsel64,
    Mipsel,
    Ppc64le,
    Ppc64,
    Ppc,
    Riscv64,
    S390x,
    S390,
}
