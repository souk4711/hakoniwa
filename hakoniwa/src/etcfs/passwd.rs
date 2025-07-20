use std::fs;
use std::path::PathBuf;

use super::typeparser::*;
use crate::error::*;

#[derive(Debug, Clone)]
pub(crate) struct PasswdEntry {
    pub(crate) name: String,
    pub(crate) uid: u32,
    pub(crate) gid: u32,
}

impl PasswdEntry {
    pub(crate) fn from_line(line: &str) -> Result<PasswdEntry> {
        let mut parts = line.split(':');

        let name = to_string(parts.next())?;
        let _password = to_string(parts.next())?;
        let uid = to_u32(parts.next())?;
        let gid = to_u32(parts.next())?;
        let _gecos = to_string(parts.next())?;
        let _directory = to_string(parts.next())?;
        let _shell = to_string(parts.next())?;

        Ok(PasswdEntry { name, uid, gid })
    }
}

#[derive(Debug, Clone)]
pub(crate) struct PasswdFile {
    pub(crate) path: PathBuf,
}

impl PasswdFile {
    pub(crate) fn new(path: &str) -> Self {
        let path = PathBuf::from(path);
        Self { path }
    }

    pub(crate) fn entries(&self) -> Result<Vec<PasswdEntry>> {
        let mut entries = vec![];
        let content = fs::read_to_string(&self.path).map_err(EtcfsErrorKind::StdIoError)?;
        for line in content.lines() {
            entries.push(PasswdEntry::from_line(line).map_err(|err| {
                EtcfsErrorKind::InvalidLine {
                    line: line[..8].to_string(),
                    errmsg: err.to_string(),
                }
            })?);
        }
        Ok(entries)
    }
}
