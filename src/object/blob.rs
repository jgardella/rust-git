use std::{fmt::Display, str::FromStr};

use crate::error::RustGitError;

use super::raw::{GitObjectRaw, GitObjectType};

pub(crate) struct GitBlobObject {
    pub(crate) contents: String,
}

impl FromStr for GitBlobObject {
    type Err = RustGitError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(GitBlobObject {
            contents: s.to_string(),
        })
    }
}

impl Display for GitBlobObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.contents)
    }
}

impl TryFrom<GitBlobObject> for GitObjectRaw {
    type Error = RustGitError;

    fn try_from(value: GitBlobObject) -> Result<Self, Self::Error> {
        return GitObjectRaw::new(GitObjectType::Blob, value.to_string());
    }
}
