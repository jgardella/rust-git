use std::collections::HashMap;
use std::fmt::Display;
use std::fs::Metadata;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use std::{env, fs};

use crate::index::{GitIndex, GitIndexEntry};
use crate::object::{
    blob::GitBlobObject, commit::GitCommitObject, id::GitObjectId, identity::GitIdentity,
    identity::GitIdentityType, raw::GitObjectRaw, raw::GitObjectType, tag::GitTagObject,
    tree::GitTreeEntry, tree::GitTreeEntryType, tree::GitTreeObject,
};
use crate::object_store::GitObjectStore;
use crate::refs::GitRefs;
use crate::{config::GitConfig, error::RustGitError};

use std::fs::File;
use std::io::Read;

const DEFAULT_GIT_DIR_NAME: &str = ".git";

const IDENTITY_ERR: &str = "*** Please tell me who you are.

Run

git config --global user.email \"you@example.com\"
git config --global user.name \"Your Name\"

to set your account's default identity.
Omit --global to set the identity only in this repository.";

pub(crate) enum RepoState {
    Repo(GitRepo),
    NoRepoExplicit(PathBuf),
    NoRepoDiscovered(PathBuf),
}

impl RepoState {
    pub(crate) fn try_get(self) -> Result<GitRepo, RustGitError> {
        match self {
            RepoState::Repo(repo) => Ok(repo),
            RepoState::NoRepoExplicit(git_dir) => Err(RustGitError::new(format!(
                "couldn't resolve provided git repository: {git_dir:?}"
            ))),
            RepoState::NoRepoDiscovered(working_dir) => Err(RustGitError::new(format!(
                "not a git repository (or any of the parent directories): {working_dir:?}"
            ))),
        }
    }
}

/// Represents a path which is relative to the root of the git repository.
#[derive(Clone, Debug, Eq)]
pub(crate) struct GitRepoPath(PathBuf);

impl GitRepoPath {
    pub fn as_path_buf(&self) -> PathBuf {
        let GitRepoPath(path_buf) = self;
        path_buf.clone()
    }

    pub fn as_string(&self) -> String {
        String::from(self.as_path_buf().to_str().unwrap())
    }

    /// Creates a new GitRepoPath which represents the current repo path moved into
    /// the provided destination.
    pub fn as_moved_file(&self, destination: &GitRepoPath) -> GitRepoPath {
        let src_file_name = self.0.file_name().unwrap();
        GitRepoPath(destination.0.join(&src_file_name))
    }

    pub fn deserialize(bytes: &[u8]) -> Result<GitRepoPath, RustGitError> {
        let path_name = String::from_utf8(bytes.to_vec())?;
        Ok(GitRepoPath(PathBuf::from(path_name)))
    }
}

impl Ord for GitRepoPath {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}

impl PartialOrd for GitRepoPath {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for GitRepoPath {
    fn eq(&self, other: &Self) -> bool {
        self.as_string() == other.as_string()
    }
}

impl Display for GitRepoPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "\"{}\"", self.as_string())
    }
}

pub(crate) struct GitRepo {
    pub(crate) config: GitConfig,
    pub(crate) index: GitIndex,
    /// Absolute path to root directory of the repo.
    pub(crate) root_dir: PathBuf,
    /// Path to working directory relative to root of repo.
    pub(crate) working_dir: PathBuf,
    /// Path to git directory of the repo.
    pub(crate) git_dir: PathBuf,
    /// Object store for repo.
    pub(crate) obj_store: GitObjectStore,
    pub(crate) refs: GitRefs,
}

impl GitRepo {
    /// Tries to find the git directory by searching upwards
    /// from the provided directory.
    fn discover_git_dir(path: &Path) -> Option<PathBuf> {
        // Simplified version of C Git implementation:
        // https://github.com/git/git/blob/master/setup.c#L1304

        let candidate_dir = path.join(DEFAULT_GIT_DIR_NAME);
        if candidate_dir.exists() {
            Some(candidate_dir)
        } else {
            Self::discover_git_dir(path.parent()?)
        }
    }

    pub fn normalize_path(&self, path: &Path) -> Option<PathBuf> {
        let mut normalized_path = PathBuf::new();

        for component in path.components() {
            match &component {
                std::path::Component::ParentDir => {
                    if !normalized_path.pop() {
                        // Path is outside repo.
                        return None;
                    }
                }
                std::path::Component::CurDir => (),
                _ => normalized_path.push(component),
            }
        }

        Some(normalized_path)
    }

    /// Converts the provided path to a canonicalized GitRepoPath relative to the root of the repo.
    /// Will return an error if the path is not within the repo.
    pub fn path_to_git_repo_path(&self, path: &Path) -> Result<GitRepoPath, RustGitError> {
        let git_repo_path = self.working_dir.join(path);
        match self.normalize_path(&git_repo_path) {
            Some(normalized_path) => Ok(GitRepoPath(normalized_path)),
            None => Err(RustGitError::new(format!(
                "path {path:?} is outside of repo"
            ))),
        }
    }

    /// Creates a new GitRepo.
    /// If git_dir is provided, it will be used to find the git directory for the repo.
    /// Otherwise, we will search for the git directory from the current working directory.
    pub(crate) fn new(git_dir: &Option<PathBuf>) -> Result<RepoState, RustGitError> {
        let current_dir = env::current_dir()?;
        let resolved_git_dir = match git_dir {
            Some(git_dir) => {
                let git_dir = git_dir.to_path_buf();
                if !git_dir.exists() {
                    return Ok(RepoState::NoRepoExplicit(git_dir));
                }
                git_dir
            }
            None => match Self::discover_git_dir(&current_dir) {
                Some(git_dir) => git_dir,
                None => return Ok(RepoState::NoRepoDiscovered(current_dir)),
            },
        };

        let config = GitConfig::new(&resolved_git_dir)?;
        // Loading the index on every repo initialization is inefficient, as it's not always needed
        // by the command, but it's simple for now.
        let index = GitIndex::open(&resolved_git_dir)?;

        let root_dir = resolved_git_dir.parent().unwrap().canonicalize()?;
        let abs_root_dir = root_dir.canonicalize()?;
        let working_dir = current_dir.strip_prefix(&abs_root_dir)?.to_path_buf();
        let obj_store = GitObjectStore::new(&resolved_git_dir);
        let refs = GitRefs::new(&resolved_git_dir)?;

        Ok(RepoState::Repo(GitRepo {
            config,
            index,
            root_dir: abs_root_dir,
            working_dir,
            git_dir: resolved_git_dir,
            obj_store,
            refs,
        }))
    }

    pub fn hash_obj(
        &self,
        obj_type: GitObjectType,
        contents: String,
        write: bool,
    ) -> Result<GitObjectId, RustGitError> {
        let obj = GitObjectRaw::new(obj_type, contents)?;

        if write {
            self.obj_store.write_raw_object(&obj)?;
        }

        Ok(obj.id)
    }

    pub(crate) fn index_path(
        &mut self,
        path: &Path,
        metadata: &Metadata,
    ) -> Result<GitObjectId, RustGitError> {
        if metadata.is_file() {
            let mut file = File::open(path)?;
            let mut contents = String::new();
            file.read_to_string(&mut contents)?;
            let blob_obj = GitBlobObject { contents };
            return self.obj_store.write_object(blob_obj);
        } else if metadata.is_symlink() {
            todo!("handle links");
        } else if metadata.is_dir() {
            todo!("handle dirs")
        }

        Err(RustGitError::new("Unsupported file type"))
    }

    /// Converts path to absolute path within this repo.
    fn path_in_repo(&self, path: &GitRepoPath) -> PathBuf {
        self.root_dir.join(path.as_string())
    }

    /// Write file relative to root of repo.
    pub(crate) fn write_file(
        &self,
        path: &GitRepoPath,
        contents: impl AsRef<[u8]>,
    ) -> Result<(), RustGitError> {
        let path_in_repo = self.path_in_repo(path);
        Ok(fs::write(path_in_repo, contents)?)
    }

    /// Remove file relative to root of repo.
    pub(crate) fn remove_file(&self, path: &GitRepoPath) -> Result<(), RustGitError> {
        let path_in_repo = self.path_in_repo(path);
        Ok(fs::remove_file(path_in_repo)?)
    }

    pub(crate) fn get_ref(&self, git_ref: &str) -> Result<Option<GitObjectId>, RustGitError> {
        self.refs.try_read_ref(git_ref)
    }

    pub(crate) fn update_ref(
        &self,
        git_ref: &str,
        new_value: &str,
        old_value: Option<&str>,
    ) -> Result<(), RustGitError> {
        if !git_ref.starts_with("refs/") {
            // TODO: C Git supports other things here (e.g. symbolic links as refs),
            // but for now we just support simple refs
            return Err(RustGitError::new(format!("invalid ref {git_ref}")));
        }
        let git_ref = git_ref.trim_start_matches("refs/");
        return self.refs.update_ref(git_ref, new_value, old_value);
    }

    /// Rename file relative to root of repo.
    pub(crate) fn rename_file(
        &self,
        old_path: &GitRepoPath,
        new_path: &GitRepoPath,
    ) -> Result<(), RustGitError> {
        let old_path_in_repo = self.path_in_repo(old_path);
        let new_path_in_repo = self.path_in_repo(new_path);
        Ok(fs::rename(old_path_in_repo, new_path_in_repo)?)
    }

    /// Look up symlink metadata relative to root of repo.
    pub(crate) fn symlink_metadata(&self, path: &GitRepoPath) -> Result<Metadata, RustGitError> {
        let path_in_repo = self.path_in_repo(path);
        Ok(fs::symlink_metadata(path_in_repo)?)
    }

    pub(crate) fn write_index_as_tree_internal(
        &self,
        entries: &Vec<&GitIndexEntry>,
        offset: usize,
    ) -> Result<GitObjectId, RustGitError> {
        let mut subtrees: HashMap<String, Vec<&GitIndexEntry>> = HashMap::new();
        let mut objects: Vec<&GitIndexEntry> = Vec::new();

        // Identify sub-trees and objects to add to create tree.
        for entry in entries {
            let path_string = &entry.path_name.as_string()[offset..];

            if let Some(slash_idx) = path_string.find("/") {
                // Path still contains a slash, so this object is part of a sub-tree.
                let dir = &path_string[..slash_idx];
                if let Some(existing_entries) = subtrees.get_mut(dir) {
                    existing_entries.push(entry);
                } else {
                    subtrees.insert(dir.to_string(), vec![entry]);
                }
            } else {
                // No slash, so this object is a direct child of this tree.
                objects.push(entry);
            }
        }

        let mut tree_entries = Vec::new();

        // Recursively compute sub-trees and add contents.
        for (name, entries) in subtrees {
            let subtree_id =
                self.write_index_as_tree_internal(&entries, offset + name.len() + 1)?;
            let entry = GitTreeEntry {
                mode: "040000".to_string(),
                entry_type: GitTreeEntryType::Tree,
                obj_id: subtree_id,
                name,
            };
            tree_entries.push(entry);
        }
        // Add blob object contents.
        for object in objects {
            let name = object.path_name.as_string()[offset..].to_string();
            let entry = GitTreeEntry {
                mode: object.mode.to_string(),
                entry_type: GitTreeEntryType::Blob,
                obj_id: object.name.clone(),
                name,
            };
            tree_entries.push(entry);
        }

        let tree_obj = GitTreeObject {
            entries: tree_entries,
        };

        return self.obj_store.write_object(tree_obj);
    }

    /// Saves the current index as a tree object in the repo.
    pub(crate) fn write_index_as_tree(&self) -> Result<GitObjectId, RustGitError> {
        let all_index_entries = &self.index.iter_entries().collect();

        self.write_index_as_tree_internal(all_index_entries, 0)
    }

    pub(crate) fn write_index(&mut self) -> Result<(), RustGitError> {
        self.index.write(&self.git_dir)
    }

    // TODO:
    // - Support other timezones ; currently always returning timestamp in UTC.
    // - Support other time formats (RFC 2822, ISO 8601); currently only supporting Git internal format.
    pub(crate) fn get_timestamp(&self) -> Result<u128, RustGitError> {
        let now = SystemTime::now();
        return Ok(now.duration_since(UNIX_EPOCH)?.as_millis());
    }

    /// Writes a commit object to the repo.
    pub(crate) fn write_commit(
        &self,
        tree: &GitObjectId,
        parents: &Vec<GitObjectId>,
        message: &str,
    ) -> Result<GitObjectId, RustGitError> {
        if !self.obj_store.is_valid_object_id(&tree) {
            return Err(RustGitError::new(format!(
                "fatal: not a valid object name {}",
                tree
            )));
        }

        for parent in parents {
            if !self.obj_store.is_valid_object_id(&parent) {
                return Err(RustGitError::new(format!(
                    "fatal: not a valid object name {}",
                    parent
                )));
            }
        }

        // TODO:
        // - load user name and config based on env vars, other config sources
        // - support separate settings for author and committer
        match (&self.config.user.name, &self.config.user.email) {
            (Some(user_name), Some(user_email)) => {
                let timestamp = self.get_timestamp()?;
                let commit_obj = GitCommitObject {
                    tree_id: tree.clone(),
                    parents: parents.clone(),
                    message: message.to_string(),
                    author: GitIdentity {
                        identity_type: GitIdentityType::Author,
                        name: user_name.to_string(),
                        email: user_email.to_string(),
                        timestamp: timestamp,
                    },
                    committer: GitIdentity {
                        identity_type: GitIdentityType::Committer,
                        name: user_name.to_string(),
                        email: user_email.to_string(),
                        timestamp: timestamp,
                    },
                };

                return self.obj_store.write_object(commit_obj);
            }
            _ => Err(RustGitError::new(IDENTITY_ERR)),
        }
    }

    pub(crate) fn get_symbolic_ref(&self, ref_name: &str) -> Result<Option<String>, RustGitError> {
        self.refs.get_symbolic_ref(ref_name)
    }

    pub(crate) fn update_symbolic_ref(
        &self,
        ref_name: &str,
        new_value: &str,
    ) -> Result<(), RustGitError> {
        self.refs.update_symbolic_ref(ref_name, new_value)
    }

    pub(crate) fn delete_symbolic_ref(&self, ref_name: &str) -> Result<(), RustGitError> {
        self.refs.delete_symbolic_ref(ref_name)
    }

    pub(crate) fn read_tag(&self, tag_name: &str) -> Result<Option<String>, RustGitError> {
        self.refs.try_read_tag(tag_name)
    }

    pub(crate) fn create_annotated_tag(
        &self,
        tag_name: &str,
        object_id: &GitObjectId,
        message: &str,
    ) -> Result<(), RustGitError> {
        if let Some(target_object) = self.obj_store.read_object_raw(&object_id)? {
            match (&self.config.user.name, &self.config.user.email) {
                (Some(user_name), Some(user_email)) => {
                    let timestamp = self.get_timestamp()?;
                    let commit_obj = GitTagObject {
                        tag_name: tag_name.to_string(),
                        object_id: object_id.clone(),
                        object_type: target_object.object.header.obj_type,
                        tagger: GitIdentity {
                            identity_type: GitIdentityType::Tagger,
                            name: user_name.to_string(),
                            email: user_email.to_string(),
                            timestamp,
                        },
                        message: message.to_string(),
                    };

                    let tag_obj_id = self.obj_store.write_object(commit_obj)?;
                    self.refs.create_tag(tag_name, &tag_obj_id)
                }
                _ => Err(RustGitError::new(IDENTITY_ERR)),
            }
        } else {
            Err(RustGitError::new(format!("no object {object_id}")))
        }
    }

    pub(crate) fn create_lightweight_tag(
        &self,
        tag_name: &str,
        object_id: &GitObjectId,
    ) -> Result<(), RustGitError> {
        self.refs.create_tag(tag_name, object_id)
    }

    pub(crate) fn delete_tag(&self, tag_name: &str) -> Result<(), RustGitError> {
        self.refs.delete_tag(tag_name)
    }

    pub(crate) fn list_tags(&self) -> Result<Vec<String>, RustGitError> {
        self.refs.list_tags()
    }

    pub(crate) fn create_ref(
        &self,
        ref_name: &str,
        object_id: &GitObjectId,
    ) -> Result<(), RustGitError> {
        self.refs.create_ref(ref_name, object_id)
    }

    pub(crate) fn list_refs(&self) -> Result<Vec<String>, RustGitError> {
        self.refs.list_refs()
    }

    pub(crate) fn rename_ref(
        &self,
        old_ref_name: &str,
        new_ref_name: &str,
    ) -> Result<(), RustGitError> {
        self.refs.rename_ref(old_ref_name, new_ref_name)
    }

    pub(crate) fn delete_ref(&self, ref_name: &str) -> Result<(), RustGitError> {
        self.refs.delete_ref(ref_name)
    }

    pub(crate) fn get_head_ref(
        &self,
    ) -> Result<(Option<String>, Option<GitObjectId>), RustGitError> {
        self.refs.get_head_ref()
    }
}
