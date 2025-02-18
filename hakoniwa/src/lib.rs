//! Process isolation for Linux using namespaces, resource limits and seccomp. It
//! works by creating a new, completely empty, mount namespace where the root is
//! on a tmpfs that is invisible from the host, and will be automatically cleaned
//! up when the last process exits.
//!
//! # Quickstart
//!
//! Use [Container] struct to build a isolated environment for executing [Command].
//!
//! ```no_run
//! use nix::unistd::{Uid, Gid};
//! use hakoniwa::Container;
//!
//! let output = Container::new()           // Create new namespaces via unshare.
//!     .rootfs("/")                        // Mount necessary directories, e.g. `/bin`
//!     .tmpfsmount("/tmp")                 // Mount new tmpfs on `/tmp`
//!     .uidmap(Uid::current().as_raw())    // Custom UID in the container
//!     .gidmap(Gid::current().as_raw())    // Custom GID in the container
//!     .command("/bin/echo")               // Create Command
//!     .arg("hello")
//!     .output()
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
pub use stdio::Stdio;
