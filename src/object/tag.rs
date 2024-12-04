use std::{fmt::Display, str::FromStr};

use crate::error::RustGitError;

use super::{
    id::GitObjectId,
    identity::GitIdentity,
    raw::{GitObjectRaw, GitObjectType},
};

pub(crate) struct GitTagObject {
    pub(crate) tag_name: String,
    pub(crate) object_id: GitObjectId,
    pub(crate) object_type: GitObjectType,
    pub(crate) tagger: GitIdentity,
    pub(crate) message: String,
}

impl FromStr for GitTagObject {
    type Err = RustGitError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut lines = s.lines().peekable();
        let object_line = lines.next();
        let object_id = match object_line {
            Some(tree_line) => {
                if !tree_line.starts_with("object ") {
                    return Err(RustGitError::new("invalid tag object, missing object id"));
                }
                GitObjectId::new(tree_line.trim_start_matches("object "))
            }
            None => {
                return Err(RustGitError::new("invalid tag object, empty"));
            }
        };

        let type_line = lines.next();
        let object_type = match type_line {
            Some(tree_line) => {
                if !tree_line.starts_with("type ") {
                    return Err(RustGitError::new("invalid tag object, missing object type"));
                }
                tree_line
                    .trim_start_matches("type ")
                    .parse::<GitObjectType>()?
            }
            None => {
                return Err(RustGitError::new("invalid tag object, missing object type"));
            }
        };

        let tag_line = lines.next();
        let tag_name = match tag_line {
            Some(tree_line) => {
                if !tree_line.starts_with("tag ") {
                    return Err(RustGitError::new("invalid tag object, missing tag name"));
                }
                tree_line.trim_start_matches("tag ").to_string()
            }
            None => {
                return Err(RustGitError::new("invalid tag object, missing tag name"));
            }
        };

        let tagger_line = lines.next();
        let tagger = if let Some(tagger_line) = tagger_line {
            tagger_line.parse::<GitIdentity>()?
        } else {
            return Err(RustGitError::new("invalid tag object, missing tagger"));
        };

        lines.next();
        lines.next();

        let message = lines.collect::<String>().to_string();

        Ok(GitTagObject {
            tag_name,
            object_id,
            object_type,
            tagger,
            message,
        })
    }
}

impl Display for GitTagObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "object {}
type {}
tag {}
tagger {} <{}> {}

{}",
            self.object_id,
            self.object_type,
            self.tag_name,
            self.tagger.name,
            self.tagger.email,
            self.tagger.timestamp,
            self.message,
        )
    }
}

impl TryFrom<GitTagObject> for GitObjectRaw {
    type Error = RustGitError;

    fn try_from(value: GitTagObject) -> Result<Self, Self::Error> {
        GitObjectRaw::new(GitObjectType::Tag, value.to_string())
    }
}
