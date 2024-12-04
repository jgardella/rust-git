use std::{fs, path::Path};

use crate::{
    command::GitCommand,
    object::{blob::GitBlobObject, tree::GitTreeEntryType, tree::GitTreeObject, GitObject},
    repo::{GitRepo, RepoState},
    RustGitError,
};

use super::cli::SwitchArgs;

pub(crate) struct SwitchCommand {
    pub branch: String,
}

impl SwitchCommand {
    pub fn new(args: SwitchArgs) -> SwitchCommand {
        SwitchCommand {
            branch: args.branch,
        }
    }

    fn write_blob_obj(&self, path: &Path, blob_obj: &GitBlobObject) -> Result<(), RustGitError> {
        Ok(fs::write(&path, &blob_obj.contents)?)
    }

    fn write_tree_obj(
        &self,
        path: &Path,
        repo: &GitRepo,
        tree_obj: &GitTreeObject,
    ) -> Result<(), RustGitError> {
        for entry in &tree_obj.entries {
            let obj = repo.obj_store.read_object(&entry.obj_id)?;
            let new_path = path.join(&entry.name);

            match entry.entry_type {
                GitTreeEntryType::Blob => {
                    if let Some(GitObject::Blob(blob_obj)) = obj {
                        self.write_blob_obj(&new_path, &blob_obj)?;
                    } else {
                        return Err(RustGitError::new("expected blob object"));
                    }
                }
                GitTreeEntryType::Tree => {
                    if let Some(GitObject::Tree(tree_obj)) = obj {
                        if !new_path.is_dir() {
                            fs::create_dir(&new_path)?;
                        }
                        self.write_tree_obj(&new_path, repo, &tree_obj)?;
                    } else {
                        return Err(RustGitError::new("expected tree object"));
                    }
                }
            }
        }

        Ok(())
    }
}

impl GitCommand for SwitchCommand {
    fn execute(&self, repo_state: RepoState) -> Result<(), RustGitError> {
        let repo = repo_state.try_get()?;

        let commit_id = if let Some(commit_id) = repo.get_ref(&self.branch)? {
            commit_id
        } else {
            return Err(RustGitError::new(format!("no ref {}", self.branch)));
        };

        // Update working tree and index based on returned object id.
        let commit_obj =
            if let Some(GitObject::Commit(obj)) = repo.obj_store.read_object(&commit_id)? {
                obj
            } else {
                return Err(RustGitError::new(format!(
                    "invalid ref {} with object id {}, expected commit",
                    self.branch, commit_id,
                )));
            };

        // Load tree for commit, and recursively traverse tree, adding objects to working directory.
        // TODO: check for overwriting existing changes in working directory
        let root_tree_obj =
            if let Some(GitObject::Tree(obj)) = repo.obj_store.read_object(&commit_obj.tree_id)? {
                obj
            } else {
                return Err(RustGitError::new(format!(
                    "invalid object {}, expected tree",
                    commit_obj.tree_id,
                )));
            };
        self.write_tree_obj(&repo.working_dir, &repo, &root_tree_obj)?;

        // Point HEAD to provided branch.
        repo.update_symbolic_ref("HEAD", &format!("refs/heads/{}", self.branch))?;

        return Ok(());
    }
}
