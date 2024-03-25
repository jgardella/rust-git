use std::fs::create_dir_all;
use std::path::{Path, PathBuf};

use crate::object::{GitObject, GitObjectContents, GitObjectId, GitObjectType};
use crate::{config::GitConfig, error::RustGitError, hash::get_hasher};

use std::fs::File;
use std::io::{Read, Write};
use flate2::read::ZlibDecoder;
use flate2::{Compression, write::ZlibEncoder};

pub(crate) enum RepoState {
    Repo(GitRepo),
    NoRepo(PathBuf),
}

impl RepoState {
    pub(crate) fn try_get(self) -> Result<GitRepo, RustGitError> {
        match self {
            RepoState::Repo(repo) => Ok(repo),
            RepoState::NoRepo(git_dir) => Err(RustGitError::new(format!("not a git repository (or any of the parent directories): {git_dir:?}"))),
        }
    }
}

pub(crate) struct GitRepo {
    pub(crate) config: GitConfig,
}

impl GitRepo {
    pub(crate) fn new(dir: &Path) -> Result<RepoState, RustGitError>
    {
        let git_dir = dir.join(".git");

        if !git_dir.exists() {
            return Ok(RepoState::NoRepo(git_dir));
        }

        let config = GitConfig::new(&git_dir)?;

        Ok(RepoState::Repo(GitRepo {
            config,
        }))
    }

    fn git_dir(&self) -> PathBuf {
        Path::new(".git").to_path_buf()
    }

    pub(crate) fn loose_object_path(&self, obj_id: &GitObjectId) -> (PathBuf, PathBuf) {
        // C Git additional logic omitted:
        // https://github.com/git/git/blob/11c821f2f2a31e70fb5cc449f9a29401c333aad2/object-file.c#L436-L445 

        let (folder_name, file_name) = obj_id.folder_and_file_name();

        let obj_folder =
            self.git_dir()
                .join("objects")
                .join(folder_name);

        (obj_folder, Path::new(file_name).to_path_buf())
    }

    fn write_object(&self, obj: &GitObject) -> Result<(), RustGitError> {
        let (obj_folder, obj_file_name) = self.loose_object_path(&obj.id);

        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(obj.content.to_string().as_bytes())?;
        let compressed_bytes = encoder.finish()?;

        create_dir_all(&obj_folder)?;
        let obj_file_path = obj_folder.join(obj_file_name);
        let mut object_file = File::create(obj_file_path)?;
        object_file.write_all(&compressed_bytes)?;

        Ok(())
    }

    pub(crate) fn read_object(&self, obj_id: &GitObjectId) -> Result<Option<GitObjectContents>, RustGitError> {
        let (obj_folder, obj_file_name) = self.loose_object_path(&obj_id);
        let obj_file_path = obj_folder.join(obj_file_name);

        if !obj_file_path.exists() {
            return Ok(None);
        }

        let object_file = File::open(obj_file_path)?;

        let mut decoder = ZlibDecoder::new(object_file);
        let mut decoded = String::new();
        decoder.read_to_string(&mut decoded).unwrap();
        let obj = decoded.parse::<GitObjectContents>()?;

        Ok(Some(obj))
    }

    pub(crate) fn index(&mut self, obj_type: GitObjectType, content: String, write: bool) -> Result<GitObjectId, RustGitError> {
        // C Git has much more additional logic here, // we just implement the core indexing logic to keep things simple:
        // - C Git implementation: https://github.com/git/git/blob/master/object-file.c#L2448
        // - C Git core indexing function: https://github.com/git/git/blob/master/object-file.c#L2312

        // Omitted blob conversion: https://github.com/git/git/blob/master/object-file.c#L2312
        // Omitted hash format check: https://github.com/git/git/blob/master/object-file.c#L2335-L2343

        let mut hasher = get_hasher(self.config.extensions.objectformat);
        let obj = GitObject::new(obj_type, content, &mut hasher)?;

        if write {
            self.write_object(&obj)?;
        }

        Ok(obj.id)
    }
}
