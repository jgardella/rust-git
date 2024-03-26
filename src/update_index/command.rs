use std::io::{self, BufRead};

use crate::{command::GitCommand, object::{GitObject, GitObjectContents, GitObjectId, GitObjectType}, repo::{GitRepo, RepoState}, RustGitError};

use super::cli::UpdateIndexArgs;

pub(crate) struct UpdateIndexCommand {
    args: UpdateIndexArgs
}

impl UpdateIndexCommand {
    pub fn new(args: UpdateIndexArgs) -> UpdateIndexCommand {
        UpdateIndexCommand { args }
    }
}

impl GitCommand for UpdateIndexCommand {
    fn execute(&self, repo_state: RepoState) -> Result<(), RustGitError>
    {
        todo!()
    }
}
