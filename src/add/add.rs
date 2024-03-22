use std::{fs::{DirBuilder, self, File}, path::{PathBuf, Path}, io::Write};
use crate::{RustGitError, config::{GitConfig, CoreConfig, ExtensionsConfig}};

use super::cli::AddArgs;

pub(crate) struct AddCommand {
    args: AddArgs,

    // TODO: add base args
}

impl AddCommand {
    pub fn new(args: AddArgs) -> AddCommand {
        AddCommand {
            args
        }
    }
}

pub(crate) fn add(cmd: &AddCommand) -> Result<(), RustGitError> // TODO: figure out return type
{
    Ok(())
}
