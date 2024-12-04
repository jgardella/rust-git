use crate::{command::GitCommand, object::id::GitObjectId, repo::RepoState, RustGitError};

use super::cli::CommitTreeArgs;

pub(crate) struct CommitTreeCommand {
    tree: GitObjectId,
    parents: Vec<GitObjectId>,
    message: String,
}

impl CommitTreeCommand {
    pub fn new(args: CommitTreeArgs) -> Result<CommitTreeCommand, RustGitError> {
        let tree = args.tree.parse::<GitObjectId>()?;
        let parents = args
            .parents
            .iter()
            .map(|parent| parent.parse::<GitObjectId>())
            .collect::<Result<Vec<GitObjectId>, RustGitError>>()?;

        let message = args.messages.join("\n");

        if message.is_empty() {
            return Err(RustGitError::new("commit message cannot be empty"));
        }

        Ok(CommitTreeCommand {
            tree,
            parents,
            message,
        })
    }
}

impl GitCommand for CommitTreeCommand {
    fn execute(&self, repo_state: RepoState) -> Result<(), RustGitError> {
        let repo = repo_state.try_get()?;

        let object_id = repo.write_commit(&self.tree, &self.parents, &self.message)?;

        println!("{object_id}");

        Ok(())
    }
}
