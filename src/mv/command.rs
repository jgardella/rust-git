
use std::{fs, path::{Path, PathBuf}};

use crate::{command::GitCommand, repo::RepoState, RustGitError};

use super::cli::MvArgs;

pub(crate) struct MvCommand {
    args: MvArgs,
}

impl MvCommand {
    pub fn new(args: MvArgs) -> MvCommand {
        MvCommand {
            args
        }
    }
}

enum MvAction {
    Move { source: Vec<PathBuf>, destination: PathBuf },
    Rename { source: PathBuf, destination: PathBuf },
}

fn get_action(files: &Vec<String>) -> Result<MvAction, RustGitError> {
    match files.as_slice() {
        [] => Err(RustGitError::new("mv expects at least 2 inputs, got 0")),
        [_] => Err(RustGitError::new("mv expects at least 2 inputs, got 1")),
        [source, destination] => {
            // If the last input provided is an existing directory,
            // we assume it's a move. Otherwise, it's a rename.
            match fs::metadata(Path::new(destination)) {
                Ok(metadata) if metadata.is_dir() => 
                    Ok(MvAction::Move { source: vec![PathBuf::from(source)], destination: PathBuf::from(destination) }),
                _ => 
                    Ok(MvAction::Rename { source: PathBuf::from(source), destination: PathBuf::from(destination) }),
            }
        },
        [source @ .., destination] => {
            // Must be a move as only move supports 3 or more arguments.
            Ok(MvAction::Move { source: source.iter().map(PathBuf::from).collect(), destination: PathBuf::from(destination) })
        }
    }
}

impl GitCommand for MvCommand {
    fn execute(&self, repo_state: RepoState) -> Result<(), RustGitError>
    {
        let mut repo = repo_state.try_get()?;

        let last_file = self.args.files.last().unwrap();

        match get_action(&self.args.files)? {
            MvAction::Move { source, destination } => todo!(),
            MvAction::Rename { source, destination } => todo!(),
        }

        fs::

        Ok(())
    }
}
