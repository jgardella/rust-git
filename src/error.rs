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

impl fmt::Display for RustGitError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.error)
    }
}
