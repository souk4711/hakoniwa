/// Represents a NET rule.
#[derive(Clone, Debug)]
pub struct Rule {
    pub(crate) port: u16,
    pub(crate) access: super::Access,
}

impl std::fmt::Display for Rule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.access, self.port)
    }
}
