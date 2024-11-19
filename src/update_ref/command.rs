use crate::{command::GitCommand, repo::RepoState, RustGitError};

use super::cli::UpdateRefArgs;

pub(crate) struct UpdateRefCommand {
    args: UpdateRefArgs,
}

impl UpdateRefCommand {
    pub fn new(args: UpdateRefArgs) -> UpdateRefCommand {
        UpdateRefCommand { args }
    }
}

impl GitCommand for UpdateRefCommand {
    fn execute(&self, repo_state: RepoState) -> Result<(), RustGitError> {
        let repo = repo_state.try_get()?;

        repo.update_ref(
            &self.args.git_ref,
            &self.args.new_value,
            self.args.old_value.as_deref(),
        )
    }
}
