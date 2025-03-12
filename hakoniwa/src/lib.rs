//! Process isolation for Linux using namespaces, resource limits and seccomp. It
//! works by creating a new, completely empty, mount namespace where the root is
//! on a tmpdir, and will be automatically cleaned up when the last process exits.
//!
//! # Quickstart
//!
//! Use [Container] to build an isolated environment, and then create a [Command]
//! to execute.
//!
//! ```no_run
//! use hakoniwa::Container;
//!
//! let output = Container::new()   // Create Container with new namespaces via unshare
//!     .rootfs("/")                // Mount necessary directories, e.g. `/bin`
//!     .tmpfsmount("/tmp")         // Mount new tmpfs on `/tmp`
//!     .command("/bin/echo")       // Create Command
//!     .arg("hello")               // Configure Command
//!     .output()                   // Execute
//!     .expect("failed to execute process witnin container");
//!
//! let hello = output.stdout;
//!
//! ```
//! More examples can be found in [hakoniwa/examples](https://github.com/souk4711/hakoniwa/tree/main/hakoniwa/examples).

mod child;
mod command;
mod container;
mod error;
mod idmap;
mod mount;
mod namespace;
mod rlimit;
mod runc;
mod runctl;
mod stdio;

use idmap::IdMap;
use mount::Mount;

pub use child::{Child, ExitStatus, Output, Rusage};
pub use command::Command;
pub use container::Container;
pub use error::{Error, Result};
pub use mount::MountOptions;
pub use namespace::Namespace;
pub use rlimit::Rlimit;
pub use runctl::Runctl;
pub use stdio::Stdio;

#[cfg(feature = "seccomp")]
pub mod seccomp;
