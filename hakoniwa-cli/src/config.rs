use anyhow::Result;
use serde::Deserialize;

#[derive(Deserialize)]
pub(crate) struct CfgConfig {
    #[serde(rename = "namespaces", default)]
    pub(crate) namespaces: Vec<CfgNamespace>,
    #[serde(rename = "mounts", default)]
    pub(crate) mounts: Vec<CfgMount>,
}

#[derive(Deserialize)]
pub(crate) struct CfgNamespace {
    #[serde(rename = "type")]
    pub(crate) nstype: String,
}

#[derive(Deserialize)]
pub(crate) struct CfgMount {
    #[serde(rename = "source", default)]
    pub(crate) source: String,
    #[serde(rename = "destination", default)]
    pub(crate) destination: String,
    #[serde(rename = "type", default)]
    pub(crate) fstype: String,
    #[serde(rename = "rw", default)]
    pub(crate) rw: bool,
}

pub(crate) fn load_str(data: &str) -> Result<CfgConfig> {
    let config: CfgConfig = toml::from_str(data)?;
    Ok(config)
}
