use std::os::unix::io::RawFd;

#[derive(Default, Debug)]
pub(crate) enum StdioType {
    #[default]
    Initial,
    Inherit,
    ByteVector,
}

/// Describes what to do with a standard I/O stream for Executor.
///
/// See also [stdout](super::Executor::stdout()), [stderr](super::Executor::stderr()),
/// and [stdin](super::Executor::stdin()) methods of [Executor](super::Executor).
#[derive(Default, Debug)]
pub struct Stdio {
    pub(crate) r#type: StdioType,
    fd: RawFd,
    bytes: Vec<u8>,
}

impl Stdio {
    /// Initial value.
    pub fn initial() -> Self {
        Self {
            r#type: StdioType::Initial,
            ..Default::default()
        }
    }

    /// Inherit the current process’s stdout/stderr/stdin.
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
            ..Default::default()
        }
    }

    pub(crate) fn inherit_stderr() -> Self {
        Self {
            r#type: StdioType::Inherit,
            fd: libc::STDERR_FILENO,
            ..Default::default()
        }
    }

    pub(crate) fn inherit_stdin() -> Self {
        Self {
            r#type: StdioType::Inherit,
            fd: libc::STDIN_FILENO,
            ..Default::default()
        }
    }

    pub(crate) fn as_raw_fd(&self) -> RawFd {
        self.fd
    }

    pub(crate) fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }
}

impl From<&str> for Stdio {
    fn from(str: &str) -> Self {
        Self {
            r#type: StdioType::ByteVector,
            bytes: String::from(str).into_bytes(),
            ..Default::default()
        }
    }
}
