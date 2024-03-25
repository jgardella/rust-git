use std::io::{self, BufRead};

use crate::{command::GitCommand, object::{GitObject, GitObjectId, GitObjectType}, repo::{GitRepo, RepoState}, RustGitError};

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
            [] => {
                if args.batch {
                    Ok(CatFileCommand::ShowAll())
                } else {
                    Err(RustGitError::new(String::from("Provide --batch for batch processing from stdin")))
                }
            }
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

fn show(s: Result<String, std::io::Error>, i: usize, repo: &mut GitRepo) -> Result<GitObject, String> {
    let line = s.map_err(|e| format!("error reading input line {} ({})", i, e))?;
    let obj_id = line.parse::<GitObjectId>().map_err(|e| format!("error parsing object id {line}: ({e})"))?;
    let obj = repo.read_object(&obj_id).map_err(|e| format!("error reading object {}, ({})", obj_id, e))?;

    match obj {
        Some(content) => Ok(GitObject { id: obj_id, content: content }),
        None => Err(format!("object {} not found", obj_id)),
    }
}
impl GitCommand for CatFileCommand {
    fn execute(&self, repo_state: RepoState) -> Result<(), RustGitError>
    {
        let mut repo = repo_state.try_get()?;

        match self {
            CatFileCommand::ShowType(obj_id) => {
                let obj = repo.read_object(obj_id)?;

                match obj {
                    Some(obj) => {
                        print!("{}", obj.header.obj_type);
                    },
                    None => {
                        print!("object {} not found", obj_id);
                    }
                }
            },
            CatFileCommand::ShowSize(obj_id) => {
                let obj = repo.read_object(obj_id)?;

                match obj {
                    Some(obj) => {
                        print!("{}", obj.header.size);
                    },
                    None => {
                        print!("object {} not found", obj_id);
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
                        print!("{}", obj.content);
                    },
                    None => {
                        print!("object {} not found", obj_id);
                    }
                }
            },
            // TODO: how to use obj_type?
            CatFileCommand::ShowContent(_, obj_id) => {
                let obj = repo.read_object(obj_id)?;

                match obj {
                    Some(obj) => {
                        print!("{}", obj.content);
                    },
                    None => {
                        print!("object {} not found", obj_id);
                    }
                }
            }
            CatFileCommand::ShowAll() => {
                for (i, line) in io::stdin().lock().lines().enumerate() {
                    match show(line, i, &mut repo) {
                        Ok(obj) => 
                            println!("{} {} {}\n{}\n", obj.id, obj.content.header.obj_type, obj.content.header.size, obj.content.content),
                        Err(err) => 
                            eprint!("{}", err),
                    }
                }
            }
        };

        Ok(())
    }
}
