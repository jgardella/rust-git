
use std::fs;

use crate::{command::GitCommand, repo::RepoState, RustGitError};

use super::cli::MvArgs;

pub(crate) struct MvCommand {
    args: MvArgs,

    // TODO: add base args
}

impl MvCommand {
    pub fn new(args: MvArgs) -> MvCommand {
        MvCommand {
            args
        }
    }
}

impl GitCommand for MvCommand {
    fn execute(&self, repo_state: RepoState) -> Result<(), RustGitError>
    {
        let mut repo = repo_state.try_get()?;

        Ok(())
    }
}
