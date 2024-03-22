use std::{fs::{DirBuilder, self, File}, path::{PathBuf, Path}, io::Write};
use crate::{RustGitError, config::{GitConfig, CoreConfig, ExtensionsConfig}};

use super::cli::HashObjectArgs;

pub(crate) struct HashObjectCommand {
    args: HashObjectArgs,

    // TODO: add base args
}

impl HashObjectCommand {
    pub fn new(args: HashObjectArgs) -> HashObjectCommand {
        HashObjectCommand {
            args
        }
    }
}

pub(crate) fn hash_object(cmd: &HashObjectCommand) -> Result<(), RustGitError> // TODO: figure out return type
{
    Ok(())
}
