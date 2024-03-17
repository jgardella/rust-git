use std::fmt;

#[derive(Debug)]
pub(crate) struct RustGitError {
    error: String
}

impl RustGitError {
    pub fn new(s: String) -> RustGitError {
        RustGitError { error: s }
    }
}

impl From<std::io::Error> for RustGitError {
    fn from(value: std::io::Error) -> Self {
        Self::new(format!("{value:?}"))
    }
}

impl std::error::Error for RustGitError {}

impl fmt::Display for RustGitError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.error)
    }
}
