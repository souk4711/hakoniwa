use std::os::unix::io::{AsRawFd, RawFd};

#[derive(Default)]
pub enum StdioType {
    #[default]
    Initial,
    Inherit,
}

#[derive(Default)]
pub struct Stdio {
    pub(crate) r#type: StdioType,
    fd: RawFd,
}

impl Stdio {
    pub fn initial() -> Self {
        Self {
            r#type: StdioType::Initial,
            ..Default::default()
        }
    }

    pub fn inherit_stdout() -> Self {
        Self {
            r#type: StdioType::Inherit,
            fd: libc::STDOUT_FILENO,
        }
    }

    pub fn inherit_stderr() -> Self {
        Self {
            r#type: StdioType::Inherit,
            fd: libc::STDERR_FILENO,
        }
    }
}

impl AsRawFd for Stdio {
    fn as_raw_fd(&self) -> RawFd {
        self.fd
    }
}
