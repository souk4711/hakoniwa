//! Process isolation for Linux using namespaces, resource limits, landlock and seccomp.
//! It works by creating a new, completely empty, mount namespace where the root is
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
//! More details can be found in [repo](https://github.com/souk4711/hakoniwa/tree/main/hakoniwa).

mod child;
mod command;
mod container;
mod error;
mod rlimit;
mod runc;
mod runctl;
mod stdio;
mod unshare;

use unshare::{FsOperation, IdMap, Mount};

pub use child::{Child, ExitStatus, Output, Rusage};
pub use command::Command;
pub use container::Container;
pub use error::{Error, Result};
pub use rlimit::Rlimit;
pub use runctl::Runctl;
pub use stdio::Stdio;
pub use unshare::{MountOptions, Namespace, Network, Pasta};

#[cfg(feature = "landlock")]
pub mod landlock;

#[cfg(feature = "seccomp")]
pub mod seccomp;
