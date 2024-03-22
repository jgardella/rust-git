use crate::{add::add::AddCommand, init::init::InitCommand, Cli, CliCommand};

pub(crate) enum Command {
    Init(InitCommand),
    Add(AddCommand),
}

// Here we have the mapping logic for converting a `CliCommand` to
// the `Command` to pass into the underlying implementation.
//
// This allows us to only pass in the base options that each command
// actually cares about.
impl From<Cli> for Command {
    fn from(value: Cli) -> Self {
        match value.command {
            CliCommand::Init(args) => 
                Command::Init(InitCommand::new(args, value.git_dir, value.work_tree)),
            CliCommand::Add(args) => 
                Command::Add(AddCommand::new(args)),
        }
    }
}
