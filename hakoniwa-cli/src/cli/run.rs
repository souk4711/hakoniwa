use anyhow::Result;
use clap::Args;

#[derive(Args)]
pub(crate) struct RunCommand {}

impl RunCommand {
    pub(crate) fn execute(&self) -> Result<()> {
        Ok(())
    }
}
