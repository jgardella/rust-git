
use std::{fs, path::{PathBuf, MAIN_SEPARATOR_STR}};

use crate::{command::GitCommand, index::GitIndexStageFlag, repo::{GitRepo, RepoState}, RustGitError};

use super::cli::RestoreArgs;

pub(crate) struct RestoreCommand {
    args: RestoreArgs,
}

impl RestoreCommand {
    pub fn new(args: RestoreArgs) -> RestoreCommand {
        RestoreCommand {
            args
        }
    }
}

impl GitCommand for RestoreCommand {
    fn execute(&self, repo_state: RepoState) -> Result<(), RustGitError>
    {
        let mut repo = repo_state.try_get()?;

        Ok(())
    }
}
