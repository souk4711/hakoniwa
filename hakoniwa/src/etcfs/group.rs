use std::fs;
use std::path::PathBuf;

use super::typeparser::*;
use crate::error::*;

#[derive(Debug, Clone)]
pub(crate) struct GroupEntry {
    pub(crate) name: String,
    pub(crate) gid: u32,
    pub(crate) members: Vec<String>,
}

impl GroupEntry {
    pub(crate) fn from_line(line: &str) -> Result<GroupEntry> {
        let mut parts = line.split(':');

        let name = to_string(parts.next())?;
        let _password = to_string(parts.next())?;
        let gid = to_u32(parts.next())?;
        let members = to_string(parts.next())?
            .split(',')
            .map(|e| e.to_string())
            .collect::<Vec<_>>();

        Ok(GroupEntry { name, gid, members })
    }
}

#[derive(Debug, Clone)]
pub(crate) struct GroupFile {
    pub(crate) path: PathBuf,
}

impl GroupFile {
    pub(crate) fn new(path: &str) -> Self {
        let path = PathBuf::from(path);
        Self { path }
    }

    pub(crate) fn entries(&self) -> Result<Vec<GroupEntry>> {
        let mut entries = vec![];
        let content = fs::read_to_string(&self.path).map_err(EtcfsErrorKind::StdIoError)?;
        for line in content.lines() {
            entries.push(GroupEntry::from_line(line).map_err(|err| {
                EtcfsErrorKind::InvalidLine {
                    line: line[..8].to_string(),
                    errmsg: err.to_string(),
                }
            })?);
        }
        Ok(entries)
    }
}
