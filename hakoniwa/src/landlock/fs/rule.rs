/// Represents a FS rule.
#[derive(Clone, Debug)]
pub struct Rule {
    pub(crate) path: String,
    pub(crate) mode: super::Access,
}

impl std::fmt::Display for Rule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.mode, self.path)
    }
}
