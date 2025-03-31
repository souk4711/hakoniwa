//! Configure landlock profile.

mod fs;
mod net;
mod ruleset;

pub use fs::Access as FsAccess;
pub use fs::Rule as FsRule;
pub use net::Access as NetAccess;
pub use net::Rule as NetRule;
pub use ruleset::{CompatMode, Resource, Ruleset};
