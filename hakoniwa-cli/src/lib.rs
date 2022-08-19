//! Process isolation for Linux using namespaces, resource limits and seccomp. It
//! works by creating a new, completely empty, mount namespace where the root is
//! on a tmpfs that is invisible from the host, and will be automatically cleaned
//! up when the last process exits. You can then use a policy configuration file or
//! commandline options to construct the root filesystem and process environment
//! and command to run in the namespace.
//!
//! More information can be found in [homepage](https://github.com/souk4711/hakoniwa).
mod cli;
mod contrib;
mod embed;
mod error;

use embed::Embed;
use error::{Error, Result};

pub use cli::execute;
