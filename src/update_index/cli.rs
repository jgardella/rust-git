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
    add: bool,

    /// If a specified file is in the index but is missing then it’s removed. Default behavior is to ignore removed file.
    remove: bool,

    /// Looks at the current index and checks to see if merges or updates are needed by checking stat() information.
    refresh: bool,

    /// Quiet. If --refresh finds that the index needs an update, the default behavior is to error out. This option makes git update-index
    /// continue anyway. 
    quiet: bool,

    /// Set the execute permissions on the updated files.
    chmod: ChmodFlag,

    /// Files to act on. Note that files beginning with . are discarded. This includes ./file and dir/./file. If you don’t want this, then
    /// use cleaner names. The same applies to directories ending / and paths with //
    #[arg(value_name="file", last=true)]
    files: Vec<String>,
}
