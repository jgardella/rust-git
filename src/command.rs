use crate::{init::init::InitCommand, CliCommand, Cli};

pub(crate) enum Command {
    Init(InitCommand)
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
                Command::Init(InitCommand::new(args, value.git_dir, value.work_tree))
        }
        
    }
}
