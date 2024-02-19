use serde::Deserialize;
use std::path::{Path, PathBuf};

#[derive(Deserialize, Clone, Default, Debug)]
#[serde(deny_unknown_fields)]
pub(crate) struct File {
    #[serde(rename = "target")]
    pub(crate) container_path: PathBuf,
    #[serde(rename = "contents")]
    pub(crate) contents: String,
}

impl File {
    pub(crate) fn new<P: AsRef<Path>>(container_path: P, contents: &str) -> Self {
        Self {
            container_path: container_path.as_ref().to_path_buf(),
            contents: contents.to_string(),
        }
    }
}
