use clap::Args;

#[derive(Args, Debug)]
#[command(about = "Move or rename a file, a directory, or a symlink")]
#[command(long_about = "
Move or rename a file, directory or symlink.

    git mv [-v] [-f] [-n] [-k] <source> <destination>
    git mv [-v] [-f] [-n] [-k] <source> ... <destination directory>

In the first form, it renames <source>, which must exist and be either a file, symlink or directory,
to <destination>. In the second form, the last argument has to be an existing directory; the given
sources will be moved into this directory.

The index is updated after successful completion, but the change must still be committed.")]
pub(crate) struct MvArgs {
    #[arg(num_args(1..))]
    files: Vec<String>,

    /// Force renaming or moving of a file even if the target exists
    #[arg(short, long)]
    force: bool,

    /// Skip move or rename actions which would lead to an error condition. An error happens when a source
    /// is neither existing nor controlled by Git, or when it would overwrite an existing file unless -f
    /// is given.
    #[arg(short('k'))]
    skip: bool,

    /// Do nothing; only show what would happen
    #[arg(short('n'), long)]
    dry_run: bool,

    /// Report the names of files as they are moved.
    #[arg(short, long)]
    verbose: bool,
}
