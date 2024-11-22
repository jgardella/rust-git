use std::{fmt::Display, str::FromStr};

use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};

use crate::{error::RustGitError, hash::Hasher};

const MAX_HEADER_LEN: usize = 32;

#[derive(Clone, Copy, Debug, PartialEq, ValueEnum)]
pub(crate) enum GitObjectType {
    Commit,
    Tree,
    Blob,
    Tag,
}

impl Display for GitObjectType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            GitObjectType::Commit => "commit",
            GitObjectType::Tree => "tree",
            GitObjectType::Blob => "blob",
            GitObjectType::Tag => "tag",
        };
        write!(f, "{}", s)
    }
}

impl FromStr for GitObjectType {
    type Err = RustGitError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "commit" => Ok(GitObjectType::Commit),
            "tree" => Ok(GitObjectType::Tree),
            "blob" => Ok(GitObjectType::Blob),
            "tag" => Ok(GitObjectType::Tag),
            _ => Err(RustGitError::new(format!(
                "'{s}' is not a valid object type"
            ))),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) struct GitObjectId(String);

impl GitObjectId {
    pub(crate) fn new(s: String) -> Self {
        GitObjectId(s)
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

#[derive(Debug, PartialEq)]
pub(crate) struct GitObjectHeader {
    pub(crate) obj_type: GitObjectType,
    pub(crate) size: usize,
}

impl Display for GitObjectHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let header = format!("{} {}\0", self.obj_type, self.size);
        let header_len = header.len();

        if header_len > MAX_HEADER_LEN {
            return Err(std::fmt::Error);
        }

        write!(f, "{}", header)
    }
}

impl FromStr for GitObjectHeader {
    type Err = RustGitError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some((obj_type, size)) = s.split_once(' ') {
            let obj_type = obj_type.parse::<GitObjectType>()?;
            let size = size.parse::<usize>()?;
            Ok(GitObjectHeader { obj_type, size })
        } else {
            Err(RustGitError::new(String::from(
                "Missing space in object header",
            )))
        }
    }
}

#[derive(Debug, PartialEq)]
pub(crate) struct GitObjectContents {
    pub(crate) header: GitObjectHeader,
    pub(crate) content: String,
}

#[derive(Debug, PartialEq)]
pub(crate) struct GitObject {
    pub(crate) id: GitObjectId,
    pub(crate) content: GitObjectContents,
}

impl Display for GitObjectContents {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.header, self.content)
    }
}

impl FromStr for GitObjectContents {
    type Err = RustGitError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some((header, content)) = s.split_once('\0') {
            let header = header.parse::<GitObjectHeader>()?;

            let content_len = content.len();

            if header.size != content_len {
                return Err(RustGitError::new(format!(
                    "Header size {} didn't match content length {}",
                    header.size, content_len
                )));
            }

            Ok(GitObjectContents {
                header,
                content: content.to_string(),
            })
        } else {
            Err(RustGitError::new(String::from(
                "Missing '\\0' in object file",
            )))
        }
    }
}

impl GitObject {
    fn get_object_id(header: &GitObjectHeader, content: &str) -> GitObjectId {
        let mut hasher = Sha1::new();
        hasher.update_fn(&header.to_string());
        hasher.update_fn(&content);
        hasher.final_oid_fn()
    }

    pub(crate) fn new(obj_type: GitObjectType, content: String) -> Result<GitObject, RustGitError> {
        let header = GitObjectHeader {
            obj_type,
            size: content.len(),
        };
        let header_string = header.to_string();
        let header_len = header_string.len();

        if header_len > MAX_HEADER_LEN {
            return Err(RustGitError::new(format!(
                "header of size {header_len} exceeded max size {MAX_HEADER_LEN}"
            )));
        }

        let id = Self::get_object_id(&header, &content);
        let content = GitObjectContents { header, content };

        Ok(GitObject { id, content })
    }
}

pub(crate) struct GitBlobObject {
    pub(crate) contents: String,
}

impl TryFrom<GitBlobObject> for GitObject {
    type Error = RustGitError;

    fn try_from(value: GitBlobObject) -> Result<Self, Self::Error> {
        return GitObject::new(GitObjectType::Blob, value.contents);
    }
}

pub(crate) struct GitCommitObject {
    pub(crate) tree: GitObjectId,
    pub(crate) parents: Vec<GitObjectId>,
    pub(crate) message: String,
    pub(crate) author_name: String,
    pub(crate) author_email: String,
    pub(crate) committer_name: String,
    pub(crate) committer_email: String,
    pub(crate) timestamp: u128,
}

impl TryFrom<GitCommitObject> for GitObject {
    type Error = RustGitError;

    fn try_from(value: GitCommitObject) -> Result<Self, Self::Error> {
        let mut content = String::new();

        content.push_str(&format!("tree {}\n", value.tree));

        for parent in value.parents {
            content.push_str(&format!("parent {parent}\n"));
        }

        content.push_str(&format!(
            "author {} <{}> {}\n",
            value.author_name, value.author_email, value.timestamp
        ));
        content.push_str(&format!(
            "committer {} <{}> {}\n\n",
            value.committer_name, value.committer_email, value.timestamp
        ));
        content.push_str(&value.message);

        return GitObject::new(GitObjectType::Commit, content);
    }
}

pub(crate) struct GitTreeEntry {
    pub(crate) mode: String,
    pub(crate) entry_type: String,
    pub(crate) obj_id: GitObjectId,
    pub(crate) name: String,
}

impl TryFrom<GitTreeObject> for GitObject {
    type Error = RustGitError;

    fn try_from(value: GitTreeObject) -> Result<Self, Self::Error> {
        let mut contents = Vec::new();

        // Recursively compute sub-trees and add contents.
        for entry in value.entries {
            contents.push(format!(
                "{} {} {}\t{}",
                entry.mode, entry.entry_type, entry.obj_id, entry.name
            ));
        }

        return GitObject::new(GitObjectType::Tree, contents.join("\n"));
    }
}

pub(crate) struct GitTreeObject {
    pub(crate) entries: Vec<GitTreeEntry>,
}

pub(crate) struct GitTagObject {
    pub(crate) tag_name: String,
    pub(crate) object_id: GitObjectId,
    pub(crate) object_type: GitObjectType,
    pub(crate) tagger_name: String,
    pub(crate) tagger_email: String,
    pub(crate) timestamp: u128,
    pub(crate) message: String,
}

impl TryFrom<GitTagObject> for GitObject {
    type Error = RustGitError;

    fn try_from(value: GitTagObject) -> Result<Self, Self::Error> {
        let mut contents = String::new();

        contents.push_str(&format!("object {}\n", value.object_id));
        contents.push_str(&format!("type {}\n", value.object_type));
        contents.push_str(&format!("tag {}\n", value.tag_name));
        contents.push_str(&format!(
            "tagger {} <{}> {}\n\n",
            value.tagger_name, value.tagger_email, value.timestamp
        ));
        contents.push_str(&value.message);

        return GitObject::new(GitObjectType::Tag, contents);
    }
}

#[cfg(test)]
mod tests {
    mod git_object_type {
        use super::super::*;

        #[test]
        fn should_parse_valid_git_object_type() {
            assert_eq!("blob".parse(), Ok(GitObjectType::Blob));
            assert_eq!("commit".parse(), Ok(GitObjectType::Commit));
            assert_eq!("tree".parse(), Ok(GitObjectType::Tree));
            assert_eq!("tag".parse(), Ok(GitObjectType::Tag));
        }

        #[test]
        fn should_fail_to_parse_invalid_git_object_type() {
            assert_eq!(
                "".parse::<GitObjectType>(),
                Err(RustGitError::new("'' is not a valid object type"))
            );
            assert_eq!(
                "invalid".parse::<GitObjectType>(),
                Err(RustGitError::new("'invalid' is not a valid object type"))
            );
        }
    }

    mod git_object_header {
        use super::super::*;

        #[test]
        fn should_parse_valid_git_object_header() {
            assert_eq!(
                "blob 20".parse(),
                Ok(GitObjectHeader {
                    obj_type: GitObjectType::Blob,
                    size: 20
                })
            );
        }

        #[test]
        fn should_fail_to_parse_invalid_git_object_header() {
            assert_eq!(
                "blob not-a-number".parse::<GitObjectHeader>(),
                Err(RustGitError::new("ParseIntError { kind: InvalidDigit }"))
            );
            assert_eq!(
                "invalid 20".parse::<GitObjectHeader>(),
                Err(RustGitError::new("'invalid' is not a valid object type"))
            );
            assert_eq!(
                "blob20".parse::<GitObjectHeader>(),
                Err(RustGitError::new("Missing space in object header"))
            );
            assert_eq!(
                "blob".parse::<GitObjectHeader>(),
                Err(RustGitError::new("Missing space in object header"))
            );
            assert_eq!(
                "20".parse::<GitObjectHeader>(),
                Err(RustGitError::new("Missing space in object header"))
            );
            assert_eq!(
                "".parse::<GitObjectHeader>(),
                Err(RustGitError::new("Missing space in object header"))
            );
        }
    }

    mod git_object_contents {
        use super::super::*;

        #[test]
        fn should_parse_valid_git_object_contents() {
            assert_eq!(
                "blob 4\0test".parse(),
                Ok(GitObjectContents {
                    header: GitObjectHeader {
                        obj_type: GitObjectType::Blob,
                        size: 4
                    },
                    content: String::from("test")
                })
            );
        }

        #[test]
        fn should_fail_to_parse_invalid_git_object_contents() {
            assert_eq!(
                "blob 5\0test".parse::<GitObjectContents>(),
                Err(RustGitError::new(
                    "Header size 5 didn't match content length 4"
                ))
            );
            assert_eq!(
                "blob 4 test".parse::<GitObjectContents>(),
                Err(RustGitError::new("Missing '\\0' in object file"))
            );
            assert_eq!(
                "".parse::<GitObjectContents>(),
                Err(RustGitError::new("Missing '\\0' in object file"))
            );
        }
    }

    mod git_object {
        use super::super::*;

        #[test]
        fn should_create_new_blob_object() {
            let obj_result = GitObject::new(GitObjectType::Blob, String::from("test"));

            assert_eq!(
                obj_result,
                Ok(GitObject {
                    id: GitObjectId(String::from("30d74d258442c7c65512eafab474568dd706c430")),
                    content: GitObjectContents {
                        header: GitObjectHeader {
                            obj_type: GitObjectType::Blob,
                            size: 4
                        },
                        content: String::from("test")
                    },
                })
            );
        }
    }
}
