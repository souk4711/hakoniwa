use serde::Deserialize;

use crate::config::{Mount, Namespace};

#[derive(Deserialize)]
pub(crate) struct Config {
    #[serde(rename = "namespaces", default)]
    pub(crate) namespaces: Vec<Namespace>,
    #[serde(rename = "mounts", default)]
    pub(crate) mounts: Vec<Mount>,
}
