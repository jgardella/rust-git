
use std::fs;

use crate::{command::GitCommand, repo::RepoState, RustGitError};

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
    // TODO: implement full restore functionality after implementing commits and branches
    fn execute(&self, repo_state: RepoState) -> Result<(), RustGitError>
    {
        let repo = repo_state.try_get()?;

        for file in self.args.files.iter() {
            println!("restoring {file}");

            let index_entries = repo.index.entry_range_by_path(&file);

            for index_entry in index_entries {
                let obj = repo.read_object(&index_entry.name)?;

                match obj {
                    Some(obj) => {
                        fs::write(&index_entry.path_name, obj.content)?;
                        println!("restored {}", index_entry.path_name);
                    },
                    None => println!("obj {} missing for {}", index_entry.name, index_entry.path_name),
                }
            }
        }

        Ok(())
    }
}
