use nix::sys::time::TimeVal;
use std::time::Duration;

pub fn from_timeval_into_duration(timeval: TimeVal) -> Duration {
    let secs = timeval.tv_sec();
    let nanos = timeval.tv_usec() * 1_000;
    Duration::new(secs as u64, nanos as u32)
}
