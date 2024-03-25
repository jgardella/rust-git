use std::{fs::File, io::{self, BufRead, BufReader}};
use crate::{command::GitCommand, repo::{GitRepo, ObjectId, ObjectType}, RustGitError};

use super::cli::CatFileArgs;

pub(crate) enum CatFileCommand {
    ShowType(ObjectId),
    ShowSize(ObjectId),
    Check(ObjectId),
    Print(ObjectId),
    ShowContent(ObjectType, ObjectId),
    ShowAll(),
}

impl CatFileCommand {
    pub fn new(args: CatFileArgs) -> Result<CatFileCommand, RustGitError> {
        match args.input[..] {
            [] => Ok(CatFileCommand::ShowAll()),
            [object] => {
                let obj_id = object.parse::<ObjectId>()?;
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
                let obj_id = object.parse::<ObjectId>()?;
                let obj_type = obj_type.parse::<ObjectType>()?;
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
        match self {
            CatFileCommand::ShowType(_) => todo!(),
            CatFileCommand::ShowSize(_) => todo!(),
            CatFileCommand::Check(_) => todo!(),
            CatFileCommand::Print(_) => todo!(),
            CatFileCommand::ShowContent(_, _) => todo!(),
            CatFileCommand::ShowAll() => todo!(),
        }
    }
}
