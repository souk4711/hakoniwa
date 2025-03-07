mod config;
mod mount;
mod namespace;

pub(crate) use config::Config;
pub(crate) use mount::Mount;
pub(crate) use namespace::Namespace;

use anyhow::Result;

pub(crate) fn load_str(data: &str) -> Result<Config> {
    let config: Config = toml::from_str(data)?;
    Ok(config)
}
