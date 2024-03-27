use std::{array::TryFromSliceError, fmt, num::ParseIntError, string::FromUtf8Error};
use hex::FromHexError;

#[derive(Debug, PartialEq)]
pub(crate) struct RustGitError {
    error: String
}

impl RustGitError {
    pub fn new(s: impl Into<String>) -> RustGitError {
        RustGitError { error: s.into() }
    }
}

impl From<String> for RustGitError {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl From<std::io::Error> for RustGitError {
    fn from(value: std::io::Error) -> Self {
        Self::new(format!("{value:?}"))
    }
}

impl From<FromHexError> for RustGitError {
    fn from(value: FromHexError) -> Self {
        Self::new(format!("{value:?}"))
    }
}

impl From<toml::ser::Error> for RustGitError {
    fn from(value: toml::ser::Error) -> Self {
        Self::new(format!("{value:?}"))
    }
}

impl From<toml::de::Error> for RustGitError {
    fn from(value: toml::de::Error) -> Self {
        Self::new(format!("{value:?}"))
    }
}

impl From<ParseIntError> for RustGitError {
    fn from(value: ParseIntError) -> Self {
        Self::new(format!("{value:?}"))
    }
}

impl From<bincode::Error> for RustGitError {
    fn from(value: bincode::Error) -> Self {
        Self::new(format!("{value:?}"))
    }
}

impl From<TryFromSliceError> for RustGitError {
    fn from(value: TryFromSliceError) -> Self {
        Self::new(format!("{value:?}"))
    }
}

impl From<FromUtf8Error> for RustGitError {
    fn from(value: FromUtf8Error) -> Self {
        Self::new(format!("{value:?}"))
    }
}

impl std::error::Error for RustGitError {}

impl fmt::Display for RustGitError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.error)
    }
}
