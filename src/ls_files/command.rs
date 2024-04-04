use std::{fs::{DirBuilder, self, File}, path::{PathBuf, Path}, io::Write};
use crate::{command::GitCommand, config::{CoreConfig, ExtensionsConfig, GitConfig}, index::GitIndexEntry, repo::{GitRepo, RepoState}, RustGitError};

use super::cli::LsFilesArgs;

pub(crate) struct LsFilesCommand {
    args: LsFilesArgs,

}

impl LsFilesCommand {
    pub fn new(args: LsFilesArgs) -> LsFilesCommand {
        // With no flags, we default to showing the cached files
        if !args.stage {
            LsFilesCommand { args: LsFilesArgs { cached: true, ..args } }
        } else {
            LsFilesCommand {
                args,
            }
        }
    }
}

impl GitCommand for LsFilesCommand {
    fn execute(&self, repo_state: RepoState) -> Result<(), RustGitError> {
        let repo = repo_state.try_get()?;

        for index_entry in repo.index.iter_entries() {
            if self.args.cached || self.args.stage {
                if self.args.stage {
                    print!("{:?} {:?} {:?}\t", index_entry.mode, index_entry.name, index_entry.flags.stage)
                }

                println!("{:?}", index_entry.path_name);

                if self.args.debug {
                    println!("  ctime: {:?}", index_entry.last_data_update);
                    println!("  mtime: {:?}", index_entry.last_metadata_update);
                    println!("  dev: {:?}\tino: {:?}", index_entry.dev, index_entry.ino);
                    println!("  uid: {:?}\tgid: {:?}", index_entry.uid, index_entry.gid);
                    println!("  size: {:?}\tflags: {:?}", index_entry.file_size, index_entry.flags);
                }
            }
        }

        Ok(())
    }
}