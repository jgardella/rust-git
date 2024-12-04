use std::{fmt::Display, str::FromStr};

use itertools::Itertools;

use crate::error::RustGitError;

use super::{
    id::GitObjectId,
    raw::{GitObjectRaw, GitObjectType},
};

impl FromStr for GitTreeObject {
    type Err = RustGitError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // TODO: better parsing logic
        let entries = s
            .lines()
            .map(|line| {
                let mut c = line.chars().peekable();

                let mode = c
                    .peeking_take_while(|c| *c != ' ')
                    .collect::<String>()
                    .to_string();
                c.next();

                let entry_type = c
                    .peeking_take_while(|c| *c != ' ')
                    .collect::<String>()
                    .parse::<GitTreeEntryType>()?;
                c.next();

                let obj_id = c.peeking_take_while(|c| *c != '\t').collect::<String>();
                let obj_id = GitObjectId::new(obj_id);
                c.next();

                let name = c.collect::<String>().to_string();

                Ok(GitTreeEntry {
                    mode,
                    entry_type,
                    obj_id,
                    name,
                })
            })
            .collect::<Result<Vec<GitTreeEntry>, RustGitError>>()?;
        Ok(GitTreeObject { entries })
    }
}

#[derive(Debug, PartialEq)]
pub(crate) enum GitTreeEntryType {
    Tree,
    Blob,
}

#[derive(Debug, PartialEq)]
pub(crate) struct GitTreeEntry {
    pub(crate) mode: String,
    pub(crate) entry_type: GitTreeEntryType,
    pub(crate) obj_id: GitObjectId,
    pub(crate) name: String,
}

impl Display for GitTreeEntryType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Tree => "tree",
            Self::Blob => "blob",
        };
        write!(f, "{}", s)
    }
}

impl FromStr for GitTreeEntryType {
    type Err = RustGitError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "tree" => Ok(Self::Tree),
            "blob" => Ok(Self::Blob),
            other => Err(RustGitError::new(format!(
                "invalid tree entry type '{other}'"
            ))),
        }
    }
}

impl Display for GitTreeObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut contents = Vec::new();

        for entry in &self.entries {
            contents.push(format!(
                "{} {} {}\t{}",
                entry.mode, entry.entry_type, entry.obj_id, entry.name
            ));
        }

        write!(f, "{}", contents.join("\n"))
    }
}

impl TryFrom<GitTreeObject> for GitObjectRaw {
    type Error = RustGitError;

    fn try_from(value: GitTreeObject) -> Result<Self, Self::Error> {
        GitObjectRaw::new(GitObjectType::Tree, value.to_string())
    }
}

#[derive(Debug, PartialEq)]
pub(crate) struct GitTreeObject {
    pub(crate) entries: Vec<GitTreeEntry>,
}

#[cfg(test)]
mod tests {
    use crate::object::{id::GitObjectId, tree::GitTreeEntry, GitTreeObject};

    #[test]
    fn should_roundtrip() {
        let entries = vec![
            GitTreeEntry {
                mode: String::from("00400"),
                entry_type: crate::object::tree::GitTreeEntryType::Tree,
                obj_id: GitObjectId::new("tree-obj-id"),
                name: String::from("test_dir"),
            },
            GitTreeEntry {
                mode: String::from("123456"),
                entry_type: crate::object::tree::GitTreeEntryType::Blob,
                obj_id: GitObjectId::new("blob-obj-id"),
                name: String::from("test.txt"),
            },
        ];
        let tree_obj = GitTreeObject { entries };

        let roundtrip_obj = tree_obj.to_string().parse::<GitTreeObject>();
        assert_eq!(Ok(tree_obj), roundtrip_obj);
    }
}
