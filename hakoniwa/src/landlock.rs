//! Configure landlock profile.

mod fs;
mod ruleset;

pub use fs::Access as FsAccess;
pub use fs::Rule as FsRule;
pub use ruleset::Ruleset;
