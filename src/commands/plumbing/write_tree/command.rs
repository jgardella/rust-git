use crate::{command::GitCommand, repo::RepoState, RustGitError};

use super::cli::WriteTreeArgs;

pub(crate) struct WriteTreeCommand {}

impl WriteTreeCommand {
    pub fn new(_: WriteTreeArgs) -> WriteTreeCommand {
        WriteTreeCommand {}
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
