pub(crate) mod blob;
pub(crate) mod commit;
pub(crate) mod id;
pub(crate) mod identity;
pub(crate) mod raw;
pub(crate) mod tag;
pub(crate) mod tree;

use blob::GitBlobObject;
use commit::GitCommitObject;
use raw::{GitObjectRaw, GitObjectType};
use tag::GitTagObject;
use tree::GitTreeObject;

use crate::error::RustGitError;

pub(crate) enum GitObject {
    Blob(GitBlobObject),
    Tree(GitTreeObject),
    Commit(GitCommitObject),
    Tag(GitTagObject),
}

impl TryFrom<GitObjectRaw> for GitObject {
    type Error = RustGitError;

    fn try_from(value: GitObjectRaw) -> Result<Self, Self::Error> {
        match value.object.header.obj_type {
            GitObjectType::Commit => Ok(GitObject::Commit(
                value.object.content.parse::<GitCommitObject>()?,
            )),
            GitObjectType::Tree => Ok(GitObject::Tree(
                value.object.content.parse::<GitTreeObject>()?,
            )),
            GitObjectType::Blob => Ok(GitObject::Blob(
                value.object.content.parse::<GitBlobObject>()?,
            )),
            GitObjectType::Tag => Ok(GitObject::Tag(
                value.object.content.parse::<GitTagObject>()?,
            )),
        }
    }
}
