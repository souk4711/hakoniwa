use std::os::unix::io::{AsRawFd, RawFd};

#[derive(Default, Debug)]
pub enum StdioType {
    #[default]
    Initial,
    Inherit,
}

#[derive(Default, Debug)]
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

    pub fn inherit() -> Self {
        Self {
            r#type: StdioType::Inherit,
            ..Default::default()
        }
    }

    pub(crate) fn inherit_stdout() -> Self {
        Self {
            r#type: StdioType::Inherit,
            fd: libc::STDOUT_FILENO,
        }
    }

    pub(crate) fn inherit_stderr() -> Self {
        Self {
            r#type: StdioType::Inherit,
            fd: libc::STDERR_FILENO,
        }
    }

    pub(crate) fn inherit_stdin() -> Self {
        Self {
            r#type: StdioType::Inherit,
            fd: libc::STDIN_FILENO,
        }
    }
}

impl AsRawFd for Stdio {
    fn as_raw_fd(&self) -> RawFd {
        self.fd
    }
}
