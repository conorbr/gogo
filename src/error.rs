use std::fmt;

/// What went wrong, as far as the exit code is concerned. See docs/cli.md.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Failed,
    Usage,
    NotFound,
    BackendUnavailable,
    InvalidSpace,
}

impl Kind {
    pub fn code(self) -> i32 {
        match self {
            Kind::Failed => 1,
            Kind::Usage => 2,
            Kind::NotFound => 3,
            Kind::BackendUnavailable => 4,
            Kind::InvalidSpace => 5,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Kind::Failed => "failed",
            Kind::Usage => "usage",
            Kind::NotFound => "not_found",
            Kind::BackendUnavailable => "backend_unavailable",
            Kind::InvalidSpace => "invalid_space",
        }
    }
}

#[derive(Debug)]
pub struct Error {
    pub kind: Kind,
    pub message: String,
}

impl Error {
    pub fn new(kind: Kind, message: impl Into<String>) -> Self {
        Error { kind, message: message.into() }
    }

    pub fn failed(message: impl Into<String>) -> Self {
        Error::new(Kind::Failed, message)
    }

    pub fn usage(message: impl Into<String>) -> Self {
        Error::new(Kind::Usage, message)
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Error::new(Kind::NotFound, message)
    }

    pub fn unavailable(message: impl Into<String>) -> Self {
        Error::new(Kind::BackendUnavailable, message)
    }

    pub fn invalid(message: impl Into<String>) -> Self {
        Error::new(Kind::InvalidSpace, message)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;
