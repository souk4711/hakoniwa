// SPDX-License-Identifier: Apache-2.0 or MIT
// Copyright 2021 [rijenkii](https://github.com/rijenkii)
//
// Ref: https://github.com/rijenkii/xdg-user-rs/blob/v0.2.1/src/lib.rs

#[derive(Debug)]
pub(crate) enum Error {
    Io(std::io::Error),
    NoHome,
    Parse,
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<std::str::Utf8Error> for Error {
    fn from(_: std::str::Utf8Error) -> Self {
        Self::Parse
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Io(e) => e.fmt(f),
            Self::NoHome => write!(f, "unable to find the home directory"),
            Self::Parse => write!(f, "error while parsing"),
        }
    }
}

impl std::error::Error for Error {}
