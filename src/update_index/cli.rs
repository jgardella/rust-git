use clap::Args;

#[derive(Args, Debug)]
#[command(about = "Register file contents in the working tree to the index")]
#[command(long_about = "
Modifies the index. Each file mentioned is updated into the index and any unmerged or needs updating state is cleared.

See also git-add(1) for a more user-friendly way to do some of the most common operations on the index.
")]
pub(crate) struct UpdateIndexArgs {
}
