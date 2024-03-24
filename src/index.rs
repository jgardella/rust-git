use std::fmt::Display;

use clap::ValueEnum;

use crate::{error::RustGitError, hash::get_hasher, init::cli::HashAlgorithm, repo::GitRepo};

const MAX_HEADER_LEN: usize = 32;

#[derive(Clone, Debug, ValueEnum)]
pub(crate) enum ObjectType {
    Commit,
    Tree,
    Blob,
    Tag,
}

impl Display for ObjectType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            ObjectType::Commit => "commit",
            ObjectType::Tree => "tree",
            ObjectType::Blob => "blob",
            ObjectType::Tag => "tag",
        };
        write!(f, "{}", s)
    }
}

pub(crate) enum ObjectId {
    ObjectId(String)
}

impl ObjectId {
    pub(crate) fn new(value: String) -> Self {
        ObjectId::ObjectId(value)
    }
}

impl Display for ObjectId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ObjectId::ObjectId(value) = self;
        write!(f, "{}", value)
    }
}

fn create_object_header(object_type: &ObjectType, obj_size: usize) -> Result<String, RustGitError> {
    let header = format!("{object_type} {obj_size}\0");
    let header_len = header.len();

    if header_len > MAX_HEADER_LEN {
        return Err(RustGitError::new(format!("header of size {header_len} exceeded max size {MAX_HEADER_LEN}")));
    }

    Ok(header)
}

fn hash_object_body(hash_algo: HashAlgorithm, header: String, obj: String) -> ObjectId {
    let mut hasher = get_hasher(hash_algo);
    hasher.update_fn(header);
    hasher.update_fn(obj);
    hasher.final_oid_fn()
}

fn hash_string(hash_algo: HashAlgorithm, object_type: &ObjectType, obj: String) -> Result<ObjectId, RustGitError> {
    let header = create_object_header(object_type, obj.len())?;
    Ok(hash_object_body(hash_algo, header, obj))
}

pub(crate) fn index(object_type: &ObjectType, obj: String, write: bool, repo: &GitRepo) -> Result<ObjectId, RustGitError> {
    // C Git has much more additional logic here, // we just implement the core indexing logic to keep things simple:
    // - C Git implementation: https://github.com/git/git/blob/master/object-file.c#L2448
    // - C Git core indexing function: https://github.com/git/git/blob/master/object-file.c#L2312

    // Omitted blob conversion: https://github.com/git/git/blob/master/object-file.c#L2312
    // Omitted hash format check: https://github.com/git/git/blob/master/object-file.c#L2335-L2343
    
    let obj_id = hash_string(repo.config.extensions.objectformat, object_type, obj)?;

    if write {
        todo!("write hash to git index")
    }

    Ok(obj_id)
}
