use crate::{command::GitCommand, repo::RepoState, RustGitError};

use super::cli::CommitArgs;

pub(crate) struct CommitCommand {
    pub message: String,
}

impl CommitCommand {
    pub fn new(args: CommitArgs) -> Result<CommitCommand, RustGitError> {
        let message = args.messages.join("\n");

        if message.is_empty() {
            return Err(RustGitError::new("commit message cannot be empty"));
        }

        Ok(CommitCommand { message })
    }
}

impl GitCommand for CommitCommand {
    fn execute(&self, repo_state: RepoState) -> Result<(), RustGitError> {
        let repo = repo_state.try_get()?;

        // Write index as tree.
        let tree_id = repo.write_index_as_tree()?;

        let (current_head_branch, current_head_ref) = repo.get_head_ref()?;

        let current_head_branch = if let Some(current_head_branch) = current_head_branch {
            current_head_branch
        } else {
            return Err(RustGitError::new("couldn't load HEAD"));
        };

        let parents = current_head_ref.map_or(vec![], |current_head| vec![current_head]);

        // Create commit object with parent set to current HEAD.
        let commit_id = repo.write_commit(&tree_id, &parents, &self.message)?;

        // Update HEAD branch to point to newly created commit.
        repo.update_ref(&current_head_branch, &commit_id.to_string(), None)?;

        println!("{commit_id}");

        Ok(())
    }
}
