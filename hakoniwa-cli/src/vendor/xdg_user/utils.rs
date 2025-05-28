// SPDX-License-Identifier: Apache-2.0 or MIT
// Copyright 2021 [rijenkii](https://github.com/rijenkii)
//
// Ref: https://github.com/rijenkii/xdg-user-rs/blob/v0.2.1/src/utils.rs

use std::io::{BufRead, BufReader};
use std::path::PathBuf;

use crate::vendor::xdg_user::{Error, LineParser};

pub(crate) fn parse_file(
    mut callback: impl FnMut(&[u8], Option<PathBuf>) -> bool,
) -> Result<(), Error> {
    #[allow(deprecated)]
    let home = std::env::home_dir().ok_or(Error::NoHome)?;

    let dirs_file_path = std::env::var_os("XDG_CONFIG_HOME")
        .and_then(|e| {
            let mut path = PathBuf::from(e);
            if path.is_absolute() {
                path.push("user-dirs.dirs");
                Some(path)
            } else {
                None
            }
        })
        .unwrap_or_else(|| home.join(".config/user-dirs.dirs"));
    let dirs_file = std::fs::File::open(dirs_file_path)?;

    let mut dirs_file = BufReader::new(dirs_file);
    let mut line = Vec::new();
    while dirs_file.read_until(b'\n', &mut line)? != 0 {
        if let Some((key, val)) = LineParser::new(&mut line).parse() {
            let val = std::str::from_utf8(val)?;

            let val = if val == "$HOME/" {
                None
            } else if let Some(folder) = val.strip_prefix("$HOME/") {
                Some(home.join(folder))
            } else {
                Some(val.into())
            };

            if !callback(key, val) {
                break;
            };
        }
        line.clear();
    }

    Ok(())
}
