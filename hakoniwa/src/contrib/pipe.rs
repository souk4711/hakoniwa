use nix::unistd;
use std::os::unix::io::{AsRawFd, RawFd};

enum FdState {
    Closed = 0,
    Opened = 1,
}

struct Fd {
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
}

impl Drop for Fd {
    fn drop(&mut self) {
        self.close()
    }
}

impl AsRawFd for Fd {
    fn as_raw_fd(&self) -> RawFd {
        self.fd
    }
}

pub type Pipe = (Fd, Fd);

pub fn create() -> Result<Pipe, nix::Error> {
    unistd::pipe().map(|pipe| {
        (
            Fd::new(pipe.0, FdState::Opened),
            Fd::new(pipe.1, FdState::Opened),
        )
    })
}
