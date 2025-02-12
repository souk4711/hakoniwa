/// Describes what to do with a standard I/O stream.
#[derive(Clone, Copy)]
pub enum Stdio {
    Inherit,
    MakePipe,
}

impl Stdio {
    /// The child inherits from the corresponding parent descriptor.
    pub fn inherit() -> Self {
        Self::Inherit
    }

    /// A new pipe should be arranged to connect the parent and child processes.
    pub fn piped() -> Self {
        Self::MakePipe
    }
}
