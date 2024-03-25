use crate::{command::GitCommand, repo::RepoState, RustGitError};

use super::cli::AddArgs;

pub(crate) struct AddCommand {
    args: AddArgs,

    // TODO: add base args
}

impl AddCommand {
    pub fn new(args: AddArgs) -> AddCommand {
        AddCommand {
            args
        }
    }
}

impl GitCommand for AddCommand {
    fn execute(&self, _: RepoState) -> Result<(), RustGitError>
    {
        todo!()
    }
}
