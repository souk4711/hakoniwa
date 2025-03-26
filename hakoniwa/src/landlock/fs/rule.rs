/// Represents a fs rule.
#[derive(Clone, Debug)]
pub struct Rule {
    pub path: String,
    pub perm: super::Perm,
}
