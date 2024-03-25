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
                } else if args.mode.check {
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
    fn execute(&self, repo: &mut GitRepo) -> Result<(), RustGitError>
    {
        match self {
            CatFileCommand::ShowType(obj_id) => {
                let obj = repo.read_object(obj_id)?;

                match obj {
                    Some(obj) => {
                        println!("{}", obj.header.obj_type);
                    },
                    None => {
                        println!("object {} not found", obj_id);
                    }
                }
            },
            CatFileCommand::ShowSize(obj_id) => {
                let obj = repo.read_object(obj_id)?;

                match obj {
                    Some(obj) => {
                        println!("{}", obj.header.size);
                    },
                    None => {
                        println!("object {} not found", obj_id);
                    }
                }
            },
            CatFileCommand::Check(obj_id) => {
                let result = repo.read_object(obj_id)?;

                return match result {
                    Some(_) => Ok(()),
                    None => Err(RustGitError::new(format!("object {obj_id} not found")))
                };
            },
            CatFileCommand::Print(obj_id) => {
                let obj = repo.read_object(obj_id)?;

                match obj {
                    Some(obj) => {
                        println!("{}", obj.content);
                    },
                    None => {
                        println!("object {} not found", obj_id);
                    }
                }
            },
            // TODO: how to use obj_type?
            CatFileCommand::ShowContent(_, obj_id) => {
                let obj = repo.read_object(obj_id)?;

                match obj {
                    Some(obj) => {
                        println!("{}", obj.content);
                    },
                    None => {
                        println!("object {} not found", obj_id);
                    }
                }
            }
            CatFileCommand::ShowAll() => todo!(),
        };

        Ok(())
    }
}
