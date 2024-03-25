use std::{fmt::Display, str::FromStr};

use clap::ValueEnum;

use crate::{error::RustGitError, hash::Hasher};

const MAX_HEADER_LEN: usize = 32;

#[derive(Clone, Copy, Debug, ValueEnum)]
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

#[derive(Clone, Debug)]
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

pub(crate) struct GitObjectHeader {
    obj_type: GitObjectType,
    size: usize,
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

pub(crate) struct GitObject {
    pub(crate) id: GitObjectId,
    pub(crate) header: GitObjectHeader,
    pub(crate) content: String,
}

impl Display for GitObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.header, self.content)
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

        Ok(GitObject {
            id,
            header,
            content,
        })
    }
}
