use clap::Args;

use crate::options::ChmodFlag;

// TODO: again lots of options here, we just implement the basic ones.
// Full C Git options:
// https://github.com/git/git/blob/master/builtin/update-index.c#L937-L1040
#[derive(Args, Debug)]
#[command(about = "Register file contents in the working tree to the index")]
#[command(long_about = "
Modifies the index. Each file mentioned is updated into the index and any unmerged or needs updating state is cleared.

See also git-add(1) for a more user-friendly way to do some of the most common operations on the index.
")]
pub(crate) struct UpdateIndexArgs {
    /// If a specified file isn’t in the index already then it’s added. Default behaviour is to ignore new files.
    #[arg(long)]
    add: bool,

    /// If a specified file is in the index but is missing then it’s removed. Default behavior is to ignore removed file.
    #[arg(long)]
    remove: bool,

    /// Files to act on. Note that files beginning with . are discarded. This includes ./file and dir/./file. If you don’t want this, then
    /// use cleaner names. The same applies to directories ending / and paths with //
    #[arg(value_name="file")]
    pub(crate) files: Vec<String>,
}
