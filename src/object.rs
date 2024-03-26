use std::{fmt::Display, str::FromStr};

use clap::ValueEnum;
use serde::{Deserialize, Serialize};

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
            _ => Err(RustGitError::new(format!("'{s}' is not a valid object type")))
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct GitObjectId(String);

impl GitObjectId {
    pub(crate) fn new(s: String) -> Self {
        GitObjectId(s)
    }

    pub(crate) fn folder_and_file_name(&self) -> (&str, &str) {
        self.0.split_at(2)
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
            Ok(GitObjectHeader {
                obj_type,
                size,
            })
        } else {
            Err(RustGitError::new(String::from("Missing space in object header")))
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
                return Err(RustGitError::new(format!("Header size {} didn't match content length {}", header.size, content_len)));
            }

            Ok(GitObjectContents {
                header,
                content: content.to_string(),
            })
        } else {
            Err(RustGitError::new(String::from("Missing '\\0' in object file")))
        }
    }
}

impl GitObject {
    fn get_object_id(header: &GitObjectHeader, content: &str, hasher: &mut Box<dyn Hasher>) -> GitObjectId {
        hasher.update_fn(&header.to_string());
        hasher.update_fn(&content);
        hasher.final_oid_fn()
    }

    pub(crate) fn new(obj_type: GitObjectType, content: String, hasher: &mut Box<dyn Hasher>) -> Result<GitObject, RustGitError> {
        let header = GitObjectHeader { obj_type, size: content.len() };
        let header_string = header.to_string();
        let header_len = header_string.len();

        if header_len > MAX_HEADER_LEN {
            return Err(RustGitError::new(format!("header of size {header_len} exceeded max size {MAX_HEADER_LEN}")));
        }

        let id = Self::get_object_id(&header, &content, hasher);
        let content = GitObjectContents {
            header,
            content,
        };

        Ok(GitObject {
            id,
            content,
        })
    }
}

#[cfg(test)]
mod tests {
    mod git_object_type {
        use super::super::*;

        #[test]
        fn should_parse_valid_git_object_type()
        {
            assert_eq!("blob".parse(), Ok(GitObjectType::Blob));
            assert_eq!("commit".parse(), Ok(GitObjectType::Commit));
            assert_eq!("tree".parse(), Ok(GitObjectType::Tree));
            assert_eq!("tag".parse(), Ok(GitObjectType::Tag));
        }

        #[test]
        fn should_fail_to_parse_invalid_git_object_type()
        {
            assert_eq!("".parse::<GitObjectType>(), Err(RustGitError::new("'' is not a valid object type")));
            assert_eq!("invalid".parse::<GitObjectType>(), Err(RustGitError::new("'invalid' is not a valid object type")));
        }
    }

    mod git_object_header {
        use super::super::*;

        #[test]
        fn should_parse_valid_git_object_header()
        {
            assert_eq!("blob 20".parse(), Ok(GitObjectHeader { obj_type: GitObjectType::Blob, size: 20 }));
        }

        #[test]
        fn should_fail_to_parse_invalid_git_object_header()
        {
            assert_eq!("blob not-a-number".parse::<GitObjectHeader>(), Err(RustGitError::new("ParseIntError { kind: InvalidDigit }")));
            assert_eq!("invalid 20".parse::<GitObjectHeader>(), Err(RustGitError::new("'invalid' is not a valid object type")));
            assert_eq!("blob20".parse::<GitObjectHeader>(), Err(RustGitError::new("Missing space in object header")));
            assert_eq!("blob".parse::<GitObjectHeader>(), Err(RustGitError::new("Missing space in object header")));
            assert_eq!("20".parse::<GitObjectHeader>(), Err(RustGitError::new("Missing space in object header")));
            assert_eq!("".parse::<GitObjectHeader>(), Err(RustGitError::new("Missing space in object header")));
        }
    }

    mod git_object_contents {
        use super::super::*;

        #[test]
        fn should_parse_valid_git_object_contents()
        {
            assert_eq!("blob 4\0test".parse(), Ok(GitObjectContents {
                header: GitObjectHeader { obj_type: GitObjectType::Blob, size: 4 },
                content: String::from("test")
            }));
        }

        #[test]
        fn should_fail_to_parse_invalid_git_object_contents()
        {
            assert_eq!("blob 5\0test".parse::<GitObjectContents>(), Err(RustGitError::new("Header size 5 didn't match content length 4")));
            assert_eq!("blob 4 test".parse::<GitObjectContents>(), Err(RustGitError::new("Missing '\\0' in object file")));
            assert_eq!("".parse::<GitObjectContents>(), Err(RustGitError::new("Missing '\\0' in object file")));
        }
    }

    mod git_object {
        use sha1::{Digest, Sha1};

        use super::super::*;

        #[test]
        fn should_create_new_blob_object()
        {
            let mut hasher: Box<dyn Hasher> = Box::new(Sha1::new());
            let obj_result = GitObject::new(GitObjectType::Blob, String::from("test"), &mut hasher);

            assert_eq!(obj_result, Ok(GitObject {
                id: GitObjectId(String::from("30d74d258442c7c65512eafab474568dd706c430")),
                content: GitObjectContents {
                    header: GitObjectHeader { obj_type: GitObjectType::Blob, size: 4 },
                    content: String::from("test")
                },
            }));
        }
    }
}
