use std::{fs::File, io::{self, BufRead, BufReader}};
use crate::{command::GitCommand, repo::GitRepo, object::{GitObjectId, GitObjectType}, RustGitError};

use super::cli::CatFileArgs;

pub(crate) enum CatFileCommand {
    ShowType(GitObjectId),
    ShowSize(GitObjectId),
    Check(GitObjectId),
    Print(GitObjectId),
    ShowContent(GitObjectType, GitObjectId),
    ShowAll(),
}

impl CatFileCommand {
    pub fn new(args: CatFileArgs) -> Result<CatFileCommand, RustGitError> {
        match &args.input[..] {
            [] => Ok(CatFileCommand::ShowAll()),
            [object] => {
                let obj_id = object.parse::<GitObjectId>()?;
                if args.mode.show_type {
                    Ok(Self::ShowType(obj_id))
                } else if args.mode.show_size {
                    Ok(Self::ShowSize(obj_id))
                } else if args.mode.show_size {
                    Ok(Self::Check(obj_id))
                } else if args.mode.print {
                    Ok(Self::Print(obj_id))
                } else {
                    Err(RustGitError::new(String::from("One of -t, -s, -e, -p is required if type is omitted")))
                }
            },
            [obj_type, object] => {
                let obj_id = object.parse::<GitObjectId>()?;
                let obj_type = obj_type.parse::<GitObjectType>()?;
                Ok(Self::ShowContent(obj_type, obj_id))
            }
            _ => {
                Err(RustGitError::new(format!("Unexpected number of args {} for cat-file", args.input.len())))
            }
        }
    }
}

impl GitCommand for CatFileCommand {
    fn execute(&self, repo: &mut GitRepo) -> Result<(), RustGitError> // TODO: figure out return type
    {
        todo!()
    }
}
