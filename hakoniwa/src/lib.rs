//! Process isolation for Linux using namespaces, resource limits and seccomp. It
//! works by creating a new, completely empty, mount namespace where the root is
//! on a tmpfs that is invisible from the host, and will be automatically cleaned
//! up when the last process exits. You can then use a policy configuration file or
//! commandline options to construct the root filesystem and process environment
//! and command to run in the namespace.
//!
//! More information can be found in [homepage](https://github.com/souk4711/hakoniwa).
//!
//! # Examples
//!
//! ```no_run
//! use hakoniwa::{Error, ExecutorResultStatus, Sandbox, SandboxPolicy, Stdio};
//!
//! fn main() -> Result<(), Error> {
//!     let mut sandbox = Sandbox::new();
//!     sandbox.with_policy(SandboxPolicy::from_str(
//!         r#"
//! mounts = [
//!   { source = "/bin"  , target = "/bin"  },
//!   { source = "/lib"  , target = "/lib"  },
//!   { source = "/lib64", target = "/lib64"},
//!   { source = "/usr"  , target = "/usr"  },
//! ]
//!     "#,
//!     )?);
//!
//!     // Killed in 2s.
//!     let prog = "sleep";
//!     let argv = vec![prog, "5"];
//!     let mut executor = sandbox.command(prog, &argv);
//!     let result = executor
//!         .limit_as(Some(16_000_000)) // 16MB
//!         .limit_core(Some(0)) // no core file
//!         .limit_fsize(Some(0)) // no output file
//!         .limit_nofile(Some(32)) // 32 max fd
//!         .limit_walltime(Some(2)) // 2 seconds
//!         .stdout(Stdio::inherit())
//!         .stderr(Stdio::inherit())
//!         .stdin(Stdio::inherit())
//!         .run();
//!     assert_eq!(result.status, ExecutorResultStatus::TimeLimitExceeded);
//!     assert_eq!(result.exit_code, Some(128 + 9));
//!
//!     Ok(())
//! }
//! ```
//!
//! More examples can be found in [hakoniwa/examples](https://github.com/souk4711/hakoniwa/tree/main/hakoniwa/examples).

mod child_process;
mod contrib;
mod error;
mod executor;
mod file;
mod idmap;
mod limits;
mod mount;
mod namespaces;
mod sandbox;
mod seccomp;
mod stdio;

use file::File;
use idmap::IDMap;
use limits::Limits;
use mount::Mount;
use namespaces::Namespaces;
use seccomp::Seccomp;
use stdio::StdioType;

pub use error::{Error, Result};
pub use executor::{Executor, ExecutorResult, ExecutorResultStatus};
pub use sandbox::{Sandbox, SandboxPolicy};
pub use seccomp::SeccompAction;
pub use stdio::Stdio;
