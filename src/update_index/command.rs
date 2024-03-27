use std::fs::{self, Metadata};

use crate::{command::GitCommand, index::GitIndexEntry, repo::{GitRepo, RepoState}, RustGitError};

use super::cli::UpdateIndexArgs;

pub(crate) struct UpdateIndexCommand {
    args: UpdateIndexArgs
}

impl UpdateIndexCommand {
    pub fn new(args: UpdateIndexArgs) -> UpdateIndexCommand {
        UpdateIndexCommand { args }
    }
}

fn process_path(path: &str, repo: &mut GitRepo) -> Result<(), RustGitError> {
    let metadata = fs::metadata(path)?;

    if metadata.is_dir() {
        todo!("process directory path");
    }

    add_one_path(path, metadata, repo)
}

fn add_one_path(path: &str, metadata: Metadata, repo: &mut GitRepo) -> Result<(), RustGitError> {
    // Write object file.
    let obj_id = repo.index_path(path, &metadata)?;

    // Make new index entry.
    let index_entry = GitIndexEntry::new(path, &metadata, obj_id);

    // Update index.
    repo.index.add(index_entry);

    Ok(())
}

impl GitCommand for UpdateIndexCommand {
    fn execute(&self, repo_state: RepoState) -> Result<(), RustGitError>
    {
        let mut repo = repo_state.try_get()?;

        for file_path in &self.args.files {
            process_path(file_path, &mut repo)?;
        }

        repo.write_index()?;

        Ok(())
    }
}
