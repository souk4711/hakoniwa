//! Runs the ownership regression against upstream's fork execution path.
//!
//! The shared test body also runs against the complete local fork.

#[path = "support/ownership_parity.rs"]
mod shared;

fn configure_command(_command: &mut hakoniwa::Command) -> std::io::Result<()> {
    Ok(())
}
