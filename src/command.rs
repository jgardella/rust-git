use crate::{add::command::AddCommand, cat_file::command::CatFileCommand, error::RustGitError, hash_object::command::HashObjectCommand, init::command::InitCommand, repo::RepoState, Cli, CliCommand};

pub(crate) trait GitCommand {
    fn execute(&self, repo_state: RepoState) -> Result<(), RustGitError>;
}

// Here we have the mapping logic for converting a `CliCommand` to
// the `Command` to pass into the underlying implementation.
//
// This allows us to only pass in the base options that each command
// actually cares about.
pub(crate) fn from_cli(value: Cli) -> Result<Box<dyn GitCommand>, RustGitError> {
    match value.command {
        CliCommand::Init(args) => 
            Ok(Box::new(InitCommand::new(args, value.git_dir, value.work_tree))),
        CliCommand::Add(args) => 
            Ok(Box::new(AddCommand::new(args))),
        CliCommand::HashObject(args) => 
            Ok(Box::new(HashObjectCommand::new(args))),
        CliCommand::CatFile(args) => 
            CatFileCommand::new(args).map(|res| Box::new(res) as Box<dyn GitCommand>)
    }
}
