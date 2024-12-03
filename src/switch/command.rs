use crate::{command::GitCommand, object::GitObject, repo::RepoState, RustGitError};

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

        println!("{}", commit_obj.tree_id);

        // Load tree for commit, and recursively traverse tree, adding objects to working directory.
        return Ok(());
    }
}
