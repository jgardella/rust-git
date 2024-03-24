use std::fs::create_dir_all;
use std::path::{Path, PathBuf};

use crate::{config::GitConfig, error::RustGitError, hash::get_hasher, init::cli::HashAlgorithm};

use std::{fmt::Display, fs::File};
use std::io::Write;
use flate2::{Compression, write::ZlibEncoder};

use clap::ValueEnum;

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

pub(crate) struct ObjectId(String);

impl ObjectId {
    pub(crate) fn new(s: String) -> Self {
        ObjectId(s)
    }

    pub(crate) fn folder_and_file_name(&self) -> (&str, &str) {
        self.0.split_at(2)
    }
}

impl Display for ObjectId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}


pub(crate) struct GitRepo {
    pub(crate) config: GitConfig,
}

impl GitRepo {
    pub(crate) fn new(dir: &Path) -> Result<GitRepo, RustGitError>
    {
        let config = GitConfig::new(dir)?;

        Ok(GitRepo {
            config,
        })
    }

    fn git_dir(&self) -> PathBuf {
        Path::new(".git").to_path_buf()
    }

    pub(crate) fn loose_object_path(&self, obj_id: &ObjectId) -> (PathBuf, PathBuf) {
        // C Git additional logic omitted:
        // https://github.com/git/git/blob/11c821f2f2a31e70fb5cc449f9a29401c333aad2/object-file.c#L436-L445 

        let (folder_name, file_name) = obj_id.folder_and_file_name();

        let obj_folder =
            self.git_dir()
                .join("objects")
                .join(folder_name);

        (obj_folder, Path::new(file_name).to_path_buf())
    }

    fn create_object_header(&self, object_type: &ObjectType, obj_size: usize) -> Result<String, RustGitError> {
        let header = format!("{object_type} {obj_size}\0");
        let header_len = header.len();

        if header_len > MAX_HEADER_LEN {
            return Err(RustGitError::new(format!("header of size {header_len} exceeded max size {MAX_HEADER_LEN}")));
        }

        Ok(header)
    }

    fn hash_object(&self, hash_algo: HashAlgorithm, header: &str, obj: &str) -> ObjectId {
        let mut hasher = get_hasher(hash_algo);
        hasher.update_fn(header);
        hasher.update_fn(obj);
        hasher.final_oid_fn()
    }

    fn write_object(&self, obj_id: &ObjectId, header: &str, obj: &str) -> Result<(), RustGitError> {
        let (obj_folder, obj_file_name) = self.loose_object_path(obj_id);

        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(header.as_bytes())?;
        encoder.write_all(obj.as_bytes())?;
        let compressed_bytes = encoder.finish()?;

        create_dir_all(&obj_folder)?;
        let obj_file_path = obj_folder.join(obj_file_name);
        let mut object_file = File::create(obj_file_path)?;
        object_file.write_all(&compressed_bytes)?;

        Ok(())
    }

    pub(crate) fn index(&mut self, object_type: &ObjectType, obj: String, write: bool) -> Result<ObjectId, RustGitError> {
        // C Git has much more additional logic here, // we just implement the core indexing logic to keep things simple:
        // - C Git implementation: https://github.com/git/git/blob/master/object-file.c#L2448
        // - C Git core indexing function: https://github.com/git/git/blob/master/object-file.c#L2312

        // Omitted blob conversion: https://github.com/git/git/blob/master/object-file.c#L2312
        // Omitted hash format check: https://github.com/git/git/blob/master/object-file.c#L2335-L2343

        let header = self.create_object_header(object_type, obj.len())?;
        let obj_id = self.hash_object(self.config.extensions.objectformat, &header, &obj);

        if write {
            self.write_object(&obj_id, &header, &obj)?;
        }

        Ok(obj_id)
    }
}
