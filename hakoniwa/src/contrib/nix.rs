pub fn from_timeval_into_duration(timeval: nix::sys::time::TimeVal) -> std::time::Duration {
    let secs = timeval.tv_sec();
    let nanos = timeval.tv_usec() * 1_000;
    std::time::Duration::new(secs as u64, nanos as u32)
}
