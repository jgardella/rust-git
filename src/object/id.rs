use std::{fmt::Display, str::FromStr};

use serde::{Deserialize, Serialize};

use crate::error::RustGitError;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) struct GitObjectId(String);

impl GitObjectId {
    pub(crate) fn new(s: impl Into<String>) -> Self {
        GitObjectId(s.into())
    }

    pub(crate) fn folder_and_file_name(&self) -> (&str, &str) {
        self.0.split_at(2)
    }

    pub(crate) fn serialize(obj: &GitObjectId) -> Vec<u8> {
        hex::decode(&obj.0).unwrap()
    }

    pub(crate) fn deserialize(bytes: &[u8]) -> Result<GitObjectId, RustGitError> {
        let bytes: &[u8; 20] = bytes.try_into()?;
        let s = hex::encode(bytes);
        Ok(GitObjectId(s))
    }
}

impl Display for GitObjectId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for GitObjectId {
    type Err = RustGitError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // TODO: any validation to add here?
        Ok(GitObjectId(String::from(s)))
    }
}
