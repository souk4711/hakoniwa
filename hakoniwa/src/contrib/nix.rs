pub mod io {
    use nix::{fcntl, unistd};
    use std::os::unix::io::RawFd;

    pub enum FdState {
        Closed,
        Opened,
    }

    pub struct Fd {
        fd: RawFd,
        state: FdState,
    }

    impl Fd {
        fn new(fd: RawFd, state: FdState) -> Self {
            Self { fd, state }
        }

        pub fn close(&mut self) {
            if let FdState::Opened = self.state {
                _ = unistd::close(self.fd);
                self.state = FdState::Closed;
            }
        }

        pub fn as_raw_fd(&self) -> RawFd {
            self.fd
        }
    }

    impl Drop for Fd {
        fn drop(&mut self) {
            self.close()
        }
    }

    pub type Pipe = (Fd, Fd);

    pub fn pipe() -> Result<Pipe, nix::Error> {
        unistd::pipe2(fcntl::OFlag::O_CLOEXEC).map(|pipe| {
            (
                Fd::new(pipe.0, FdState::Opened),
                Fd::new(pipe.1, FdState::Opened),
            )
        })
    }
}

pub mod time {
    use nix::sys::time::TimeVal;
    use std::time::Duration;

    pub fn from_timeval_into_duration(timeval: TimeVal) -> Duration {
        let secs = timeval.tv_sec();
        let nanos = timeval.tv_usec() * 1_000;
        Duration::new(secs as u64, nanos as u32)
    }
}
