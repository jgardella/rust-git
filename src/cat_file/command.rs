use std::{fs::File, io::{self, BufRead, BufReader}};
use crate::{command::GitCommand, repo::GitRepo, RustGitError};

use super::cli::CatFileArgs;

pub(crate) struct CatFileCommand {
    args: CatFileArgs,

    // TODO: add base args
}

impl CatFileCommand {
    pub fn new(args: CatFileArgs) -> CatFileCommand {
        CatFileCommand {
            args
        }
    }
}

impl GitCommand for CatFileCommand {
    fn execute(&self, repo: &mut GitRepo) -> Result<(), RustGitError> // TODO: figure out return type
    {
        todo!()
    }
}
