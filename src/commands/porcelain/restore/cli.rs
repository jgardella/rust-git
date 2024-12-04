use clap::Args;

#[derive(Args, Debug)]
#[command(about = "Restore working tree files")]
#[command(long_about = "
Restore specified paths in the working tree with some contents from a restore source. If a path is
tracked but does not exist in the restore source, it will be removed to match the source.

The command can also be used to restore the content in the index with --staged, or restore both the
working tree and the index with --staged --worktree.

By default, the restore sources for working tree and the index are the index and HEAD respectively.
--source could be used to specify a commit as the restore source.

See \"Reset, restore and revert\" in git(1) for the differences between the three commands.

THIS COMMAND IS EXPERIMENTAL. THE BEHAVIOR MAY CHANGE.")]
pub(crate) struct RestoreArgs {
    // TODO: make this a pathspec
    /// Limits the paths affected by this operation.
    pub(crate) files: Vec<String>,
}
