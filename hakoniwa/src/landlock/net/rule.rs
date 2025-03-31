/// Represents a NET rule.
#[derive(Clone, Debug)]
pub struct Rule {
    pub port: u16,
    pub access: super::Access,
}
