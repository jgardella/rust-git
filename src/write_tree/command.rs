use crate::{command::GitCommand, repo::RepoState, RustGitError};

use super::cli::WriteTreeArgs;

pub(crate) struct WriteTreeCommand {
    args: WriteTreeArgs,
}

impl WriteTreeCommand {
    pub fn new(args: WriteTreeArgs) -> WriteTreeCommand {
        WriteTreeCommand { args }
    }
}

impl GitCommand for WriteTreeCommand {
    fn execute(&self, repo_state: RepoState) -> Result<(), RustGitError> {
        let repo = repo_state.try_get()?;

        let object_id = repo.write_index_as_tree()?;

        println!("{object_id}");

        Ok(())
    }
}
