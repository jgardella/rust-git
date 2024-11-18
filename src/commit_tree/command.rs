use std::str::FromStr;

use crate::{command::GitCommand, object::GitObjectId, repo::RepoState, RustGitError};

use super::cli::CommitTreeArgs;

pub(crate) struct CommitTreeCommand {
    args: CommitTreeArgs,
    tree: GitObjectId,
    parents: Vec<GitObjectId>,
    message: String,
}

impl CommitTreeCommand {
    pub fn new(args: CommitTreeArgs) -> Result<CommitTreeCommand, RustGitError> {
        let tree = GitObjectId::from_str(&args.tree)?;
        let parents = args
            .parents
            .iter()
            .map(|parent| GitObjectId::from_str(&parent))
            .collect::<Result<Vec<GitObjectId>, RustGitError>>()?;

        let message = args.messages.join("\n");

        Ok(CommitTreeCommand {
            args,
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
