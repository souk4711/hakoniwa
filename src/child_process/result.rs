use bincode::config;
use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use std::{os::unix::io::RawFd, time::Duration};

use crate::{
    child_process::{error::Error, error::Result, syscall},
    Executor, ExecutorResultStatus,
};

#[derive(Serialize, Deserialize, PartialEq, Default, Debug)]
pub struct ChildProcessResult {
    pub(crate) status: ExecutorResultStatus,
    pub(crate) reason: String,
    pub(crate) exit_code: i32,
    pub(crate) real_time: Duration,
    pub(crate) start_time: DateTime<Utc>,
}

impl ChildProcessResult {
    const DEFAULT_BUF_SIZE: usize = 8 * 1024;

    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn failure(reason: &str) -> Self {
        Self {
            status: ExecutorResultStatus::SandboxSetupError,
            reason: reason.to_string(),
            exit_code: Executor::EXITCODE_FAILURE,
            ..Default::default()
        }
    }

    pub fn send_to(writer: RawFd, cpr: Self) -> Result<()> {
        let config = config::standard();
        let encoded: Vec<u8> = match bincode::serde::encode_to_vec(&cpr, config) {
            Ok(val) => val,
            Err(err) => return Err(Error(err.to_string())),
        };
        syscall::write(writer, encoded.as_slice()).map(|_| ())
    }

    pub fn recv_from(reader: RawFd) -> Result<ChildProcessResult> {
        let mut encoded: [u8; Self::DEFAULT_BUF_SIZE] = [0; Self::DEFAULT_BUF_SIZE];
        syscall::read(reader, &mut encoded)?;

        let config = config::standard();
        let (decoded, _): (Self, usize) =
            match bincode::serde::decode_from_slice(&encoded[..], config) {
                Ok(val) => val,
                Err(err) => return Err(Error(err.to_string())),
            };
        Ok(decoded)
    }
}
