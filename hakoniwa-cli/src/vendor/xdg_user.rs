// SPDX-License-Identifier: Apache-2.0 or MIT
// Copyright 2021 [rijenkii](https://github.com/rijenkii)
//
// Ref: https://github.com/rijenkii/xdg-user-rs/blob/master/src/lib.rs

mod error;
mod parser;
mod utils;

use error::Error;
use parser::LineParser;

pub(crate) fn user_dir(env: &str) -> Result<Option<std::path::PathBuf>, Error> {
    let mut ret = None;
    utils::parse_file(|key, val| {
        if key == env.as_bytes() {
            ret = val;
            false
        } else {
            true
        }
    })?;
    Ok(ret)
}
