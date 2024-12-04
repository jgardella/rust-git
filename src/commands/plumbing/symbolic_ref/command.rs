use crate::{command::GitCommand, repo::RepoState, RustGitError};

use super::cli::SymbolicRefArgs;

pub(crate) struct ReadSymbolicRef {
    pub(crate) ref_name: String,
    pub(crate) quiet: bool,
    pub(crate) short: bool,
}

pub(crate) struct UpdateSymbolicRef {
    pub(crate) ref_name: String,
    pub(crate) new_value: String,
}

pub(crate) struct DeleteSymbolicRef {
    pub(crate) ref_name: String,
    pub(crate) quiet: bool,
}

pub(crate) enum SymbolicRefCommand {
    ReadSymbolicRef(ReadSymbolicRef),
    UpdateSymbolicRef(UpdateSymbolicRef),
    DeleteSymbolicRef(DeleteSymbolicRef),
}

impl SymbolicRefCommand {
    pub fn new(args: SymbolicRefArgs) -> SymbolicRefCommand {
        if let Some(new_value) = args.new_value {
            return SymbolicRefCommand::UpdateSymbolicRef(UpdateSymbolicRef {
                ref_name: args.ref_name,
                new_value: new_value,
            });
        }
        if args.delete {
            return SymbolicRefCommand::DeleteSymbolicRef(DeleteSymbolicRef {
                ref_name: args.ref_name,
                quiet: args.quiet,
            });
        }

        return SymbolicRefCommand::ReadSymbolicRef(ReadSymbolicRef {
            ref_name: args.ref_name,
            quiet: args.quiet,
            short: args.short,
        });
    }
}

impl GitCommand for SymbolicRefCommand {
    fn execute(&self, repo_state: RepoState) -> Result<(), RustGitError> {
        let repo = repo_state.try_get()?;

        match self {
            SymbolicRefCommand::ReadSymbolicRef(read_cmd) => {
                if let Some(ref_value) = repo.refs.get_symbolic_ref(&read_cmd.ref_name)? {
                    if !ref_value.starts_with("ref: ") {
                        if read_cmd.quiet {
                            return Err(RustGitError::new(""));
                        } else {
                            println!("HEAD is in detatched state");
                            return Ok(());
                        }
                    }

                    let ref_value = if read_cmd.short {
                        ref_value.trim_start_matches("ref: refs/heads/")
                    } else {
                        ref_value.trim_start_matches("ref: ")
                    };

                    println!("{ref_value}");
                } else {
                    println!("no symbolic-ref '{}'", read_cmd.ref_name);
                }
            }
            SymbolicRefCommand::UpdateSymbolicRef(update_cmd) => {
                repo.refs
                    .update_symbolic_ref(&update_cmd.ref_name, &update_cmd.new_value)?;
            }
            SymbolicRefCommand::DeleteSymbolicRef(delete_cmd) => {
                // TODO: respect quiet flag for deletions
                repo.refs.delete_symbolic_ref(&delete_cmd.ref_name)?;
            }
        }

        Ok(())
    }
}
