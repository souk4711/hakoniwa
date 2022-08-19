//! Process isolation for Linux using namespaces, resource limits and seccomp. It
//! works by creating a new, completely empty, mount namespace where the root is
//! on a tmpfs that is invisible from the host, and will be automatically cleaned
//! up when the last process exits. You can then use a policy configuration file or
//! commandline options to construct the root filesystem and process environment
//! and command to run in the namespace.
//!
//! More information can be found in [homepage](https://github.com/souk4711/hakoniwa).
mod child_process;
mod contrib;
mod error;
mod executor;
mod idmap;
mod limits;
mod mount;
mod namespaces;
mod sandbox;
mod seccomp;
mod stdio;

use idmap::IDMap;
use limits::Limits;
use mount::{Mount, MountType};
use namespaces::Namespaces;
use seccomp::Seccomp;
use stdio::StdioType;

pub use error::{Error, Result};
pub use executor::{Executor, ExecutorResult, ExecutorResultStatus};
pub use sandbox::{Sandbox, SandboxPolicy};
pub use stdio::Stdio;
