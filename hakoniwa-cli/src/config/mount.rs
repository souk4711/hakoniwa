use serde::Deserialize;

#[derive(Deserialize)]
pub(crate) struct Mount {
    #[serde(rename = "source", default)]
    pub(crate) source: String,
    #[serde(rename = "destination", default)]
    pub(crate) destination: String,
    #[serde(rename = "type", default)]
    pub(crate) fstype: String,
    #[serde(rename = "rw", default)]
    pub(crate) rw: bool,
}
