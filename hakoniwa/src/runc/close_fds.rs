use close_fds::close_open_fds;
use std::os::fd::RawFd;

use super::error::*;

pub(crate) fn close_extra_fds_exclude(reader: RawFd, writer: RawFd) -> Result<()> {
    let mut keep_fds = [reader, writer];
    keep_fds.sort_unstable();

    unsafe {
        close_open_fds(3, &keep_fds);
    }
    Ok(())
}
