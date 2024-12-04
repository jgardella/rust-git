use std::{fmt::Display, str::FromStr};

use crate::error::RustGitError;

use super::raw::{GitObjectRaw, GitObjectType};

#[derive(Debug, PartialEq)]
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

#[cfg(test)]
mod tests {
    use crate::object::GitBlobObject;

    #[test]
    fn should_roundtrip() {
        let blob_obj = GitBlobObject {
            contents: String::from("my test file contents"),
        };

        let roundtrip_obj = blob_obj.to_string().parse::<GitBlobObject>();
        assert_eq!(Ok(blob_obj), roundtrip_obj);
    }
}
