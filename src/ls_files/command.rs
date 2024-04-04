use std::{fs::{DirBuilder, self, File}, path::{PathBuf, Path}, io::Write};
use crate::{command::GitCommand, config::{CoreConfig, ExtensionsConfig, GitConfig}, repo::RepoState, RustGitError};

use super::cli::LsFilesArgs;

const DEFAULT_GIT_DIR: &str = ".git";

pub(crate) struct LsFilesCommand {
    args: LsFilesArgs,

}

impl LsFilesCommand {
    pub fn new(args: LsFilesArgs) -> LsFilesCommand {
        LsFilesCommand {
            args,
        }
    }
}


impl GitCommand for LsFilesCommand {

    fn execute(&self, _: RepoState) -> Result<(), RustGitError>
    {
        todo!()
    }
}
