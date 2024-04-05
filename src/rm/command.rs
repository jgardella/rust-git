
use crate::{command::GitCommand, repo::{GitRepo, RepoState}, RustGitError};

use super::cli::RmArgs;

pub(crate) struct RmCommand {
    args: RmArgs,

    // TODO: add base args
}

impl RmCommand {
    pub fn new(args: RmArgs) -> RmCommand {
        RmCommand {
            args
        }
    }
}

impl GitCommand for RmCommand {
    fn execute(&self, repo_state: RepoState) -> Result<(), RustGitError>
    {
        let mut repo = repo_state.try_get()?;
        todo!()
    }
}
