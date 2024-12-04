use std::{fmt::Display, str::FromStr};

use itertools::Itertools;

use crate::error::RustGitError;

use super::{
    id::GitObjectId,
    identity::{GitIdentity, GitIdentityType},
    raw::{GitObjectRaw, GitObjectType},
};

pub(crate) struct GitCommitObject {
    pub(crate) tree_id: GitObjectId,
    pub(crate) parents: Vec<GitObjectId>,
    pub(crate) message: String,
    pub(crate) author: GitIdentity,
    pub(crate) committer: GitIdentity,
}

impl FromStr for GitCommitObject {
    type Err = RustGitError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut lines = s.lines().peekable();

        let tree_line = lines.next();

        let tree_id = match tree_line {
            Some(tree_line) => {
                if !tree_line.starts_with("tree ") {
                    return Err(RustGitError::new("invalid commit object, missing tree"));
                }
                GitObjectId::new(tree_line.trim_start_matches("tree "))
            }
            None => {
                return Err(RustGitError::new("invalid commit object, empty"));
            }
        };

        let parents = lines
            .peeking_take_while(|line| line.starts_with("parent "))
            .map(|line| GitObjectId::new(line.trim_start_matches("parent ")))
            .collect::<Vec<GitObjectId>>();

        let author_line = lines.next();
        let author = if let Some(author_line) = author_line {
            author_line.parse::<GitIdentity>()?
        } else {
            return Err(RustGitError::new("invalid commit object, missing author"));
        };

        if author.identity_type != GitIdentityType::Author {
            return Err(RustGitError::new(format!(
                "invalid commit object, expected author but got {}",
                author.identity_type
            )));
        }

        let committer_line = lines.next();
        let committer = if let Some(committer_line) = committer_line {
            committer_line.parse::<GitIdentity>()?
        } else {
            return Err(RustGitError::new(
                "invalid commit object, missing committer",
            ));
        };

        if committer.identity_type != GitIdentityType::Committer {
            return Err(RustGitError::new(format!(
                "invalid commit object, expected committer but got {}",
                author.identity_type
            )));
        }

        lines.next();
        lines.next();

        let message = lines.collect::<Vec<&str>>().join("\n");

        return Ok(GitCommitObject {
            tree_id,
            parents,
            message,
            author,
            committer,
        });
    }
}

impl Display for GitCommitObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut content = String::new();

        content.push_str(&format!("tree {}\n", self.tree_id));

        for parent in &self.parents {
            content.push_str(&format!("parent {parent}\n"));
        }

        content.push_str(&format!(
            "author {} <{}> {}\n",
            self.author.name, self.author.email, self.author.timestamp
        ));
        content.push_str(&format!(
            "committer {} <{}> {}\n\n",
            self.committer.name, self.committer.email, self.committer.timestamp
        ));
        content.push_str(&self.message);

        write!(f, "{}", content)
    }
}

impl TryFrom<GitCommitObject> for GitObjectRaw {
    type Error = RustGitError;

    fn try_from(value: GitCommitObject) -> Result<Self, Self::Error> {
        GitObjectRaw::new(GitObjectType::Commit, value.to_string())
    }
}
