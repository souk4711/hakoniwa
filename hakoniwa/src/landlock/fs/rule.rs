/// Represents a fs rule.
#[derive(Clone, Debug)]
pub struct Rule {
    pub path: String,
    pub mode: super::Access,
}

impl std::fmt::Display for Rule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.mode, self.path)
    }
}
