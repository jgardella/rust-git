use crate::{add::add::AddCommand, cat_file::{cli::CatFileArgs, command::CatFileCommand}, error::RustGitError, hash_object::hash_object::HashObjectCommand, init::init::InitCommand, repo::GitRepo, Cli, CliCommand};

pub(crate) trait GitCommand {
    fn execute(&self, repo: &mut GitRepo) -> Result<(), RustGitError>;
}

// Here we have the mapping logic for converting a `CliCommand` to
// the `Command` to pass into the underlying implementation.
//
// This allows us to only pass in the base options that each command
// actually cares about.
pub(crate) fn from_cli(value: Cli) -> Box<dyn GitCommand> {
    match value.command {
        CliCommand::Init(args) => 
            Box::new(InitCommand::new(args, value.git_dir, value.work_tree)),
        CliCommand::Add(args) => 
            Box::new(AddCommand::new(args)),
        CliCommand::HashObject(args) => 
            Box::new(HashObjectCommand::new(args)),
        CliCommand::CatFile(args) => 
            Box::new(CatFileCommand::new(args)),
    }
}
