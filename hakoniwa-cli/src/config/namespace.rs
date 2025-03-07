use serde::Deserialize;

#[derive(Deserialize)]
pub(crate) struct Namespace {
    #[serde(rename = "type")]
    pub(crate) nstype: String,
}
