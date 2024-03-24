use std::path::Path;

use crate::{add::add::AddCommand, error::RustGitError, hash_object::hash_object::HashObjectCommand, init::init::InitCommand, repo::GitRepo, Cli, CliCommand};

pub(crate) enum Command {
    Init(InitCommand),
    Add(AddCommand),
    HashObject(HashObjectCommand),
}

// Here we have the mapping logic for converting a `CliCommand` to
// the `Command` to pass into the underlying implementation.
//
// This allows us to only pass in the base options that each command
// actually cares about.
impl TryFrom<Cli> for Command {
    fn try_from(value: Cli) -> Result<Self, RustGitError> {
        Ok(match value.command {
            CliCommand::Init(args) => 
                Command::Init(InitCommand::new(args, value.git_dir, value.work_tree)),
            CliCommand::Add(args) => 
                Command::Add(AddCommand::new(args)),
            CliCommand::HashObject(args) => 
                Command::HashObject(HashObjectCommand::new(args)),
        })
    }
    
    type Error = RustGitError;
}
