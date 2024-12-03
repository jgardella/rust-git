use clap::Args;

#[derive(Args, Debug)]
#[command(about = "Switch branches")]
#[command(long_about = "
Switch to a specified branch. The working tree and the index are updated to match the branch. All new commits
will be added to the tip of this branch.

Optionally a new branch could be created with either -c, -C, automatically from a remote branch of same name
(see --guess), or detach the working tree from any branch with --detach, along with switching.

Switching branches does not require a clean index and working tree (i.e. no differences compared to HEAD). The
operation is aborted however if the operation leads to loss of local changes, unless told otherwise with
--discard-changes or --merge.
")]
pub(crate) struct SwitchArgs {
    pub branch: String,
}
