use anyhow::{Result, anyhow};
use std::cmp;
use std::fs;
use std::path::{Path, PathBuf};

pub(crate) struct UserDirsEntry {
    pub(crate) name: String,
    pub(crate) value: String,
}

impl UserDirsEntry {
    fn from_line(line: &str) -> Result<Self> {
        let mut parts = line.split('=');
        let name = Self::to_string(parts.next())?;
        let value = Self::to_string(parts.next())?
            .trim_start_matches('"')
            .trim_end_matches('"')
            .to_string();
        Ok(Self { name, value })
    }

    fn to_string(option: Option<&str>) -> Result<String> {
        match option {
            Some(v) => Ok(v.to_string()),
            None => Err(anyhow!("not enough parts")),
        }
    }
}

pub(crate) struct UserDirsFile {
    path: PathBuf,
}

impl UserDirsFile {
    pub(crate) fn new(path: &Path) -> Self {
        let path = PathBuf::from(path);
        Self { path }
    }

    pub(crate) fn entries(&self) -> Result<Vec<UserDirsEntry>> {
        let mut entries = vec![];
        let content = fs::read_to_string(&self.path)?;
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with("#") {
                continue;
            }
            entries.push(UserDirsEntry::from_line(line).map_err(|err| {
                let line = line[..cmp::min(line.len(), 8)].to_string();
                anyhow!("parse line `{line}..` failed: {err}")
            })?);
        }
        Ok(entries)
    }
}
