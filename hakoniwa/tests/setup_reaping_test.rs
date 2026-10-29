//! Checks failed-startup reaping through the public fork execution path.
//!
//! Refused acquisition tests require the external syscall-refusal wrapper.

#[path = "support/setup_reaping.rs"]
mod shared;

fn configure_command(_command: &mut hakoniwa::Command) -> std::io::Result<()> {
    Ok(())
}
