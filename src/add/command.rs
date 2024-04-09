use std::{fs::{self, Metadata}, path::Path};

use crate::{command::GitCommand, index::GitIndexEntry, repo::{GitRepo, RepoState}, RustGitError};

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

fn process_path(path: &str, repo: &mut GitRepo) -> Result<(), RustGitError> {
    let metadata = fs::metadata(path)?;

    if metadata.is_dir() {
        for dir_item in fs::read_dir(path)? {
            process_path(dir_item?.path().to_str().unwrap(), repo)?
        }
    } else {
        add_one_path(path, metadata, repo)?;
    }

    Ok(())
}

fn add_one_path(path: &str, metadata: Metadata, repo: &mut GitRepo) -> Result<(), RustGitError> {
    let path = Path::new(path);
    // Write object file.
    let obj_id = repo.index_path(&path, &metadata)?;

    // Make new index entry.
    let repo_path = repo.path_to_git_repo_path(Path::new(path))?;
    let index_entry = GitIndexEntry::new(&repo_path, &metadata, obj_id);

    // Update index.
    repo.index.add(index_entry);

    Ok(())
}

impl GitCommand for AddCommand {
    fn execute(&self, repo_state: RepoState) -> Result<(), RustGitError>
    {
        let mut repo = repo_state.try_get()?;

        for file_path in &self.args.pathspec {
            process_path(file_path, &mut repo)?;
        }

        repo.write_index()?;

        Ok(())
    }
}
