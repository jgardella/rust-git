use std::path::PathBuf;

use clap::Args;

#[derive(Args, Debug)]
#[command(about = "List, create, or delete branches")]
#[command(long_about = "
If --list is given, or if there are no non-option arguments, existing branches are listed; the current branch
will be highlighted in green and marked with an asterisk. Any branches checked out in linked worktrees will be
highlighted in cyan and marked with a plus sign. Option -r causes the remote-tracking branches to be listed,
and option -a shows both local and remote branches.

If a <pattern> is given, it is used as a shell wildcard to restrict the output to matching branches. If
multiple patterns are given, a branch is shown if it matches any of the patterns.

Note that when providing a <pattern>, you must use --list; otherwise the command may be interpreted as branch
creation.

With --contains, shows only the branches that contain the named commit (in other words, the branches whose tip
commits are descendants of the named commit), --no-contains inverts it. With --merged, only branches merged
into the named commit (i.e. the branches whose tip commits are reachable from the named commit) will be
listed. With --no-merged only branches not merged into the named commit will be listed. If the <commit>
argument is missing it defaults to HEAD (i.e. the tip of the current branch).

The command’s second form creates a new branch head named <branchname> which points to the current HEAD, or
<start-point> if given. As a special case, for <start-point>, you may use \"A...B\" as a shortcut for the merge
base of A and B if there is exactly one merge base. You can leave out at most one of A and B, in which case it
defaults to HEAD.

Note that this will create the new branch, but it will not switch the working tree to it; use \"git switch
<newbranch>\" to switch to the new branch.

When a local branch is started off a remote-tracking branch, Git sets up the branch (specifically the
branch.<name>.remote and branch.<name>.merge configuration entries) so that git pull will appropriately merge
from the remote-tracking branch. This behavior may be changed via the global branch.autoSetupMerge
configuration flag. That setting can be overridden by using the --track and --no-track options, and changed
later using git branch --set-upstream-to.

With a -m or -M option, <oldbranch> will be renamed to <newbranch>. If <oldbranch> had a corresponding reflog,
it is renamed to match <newbranch>, and a reflog entry is created to remember the branch renaming. If
<newbranch> exists, -M must be used to force the rename to happen.

The -c and -C options have the exact same semantics as -m and -M, except instead of the branch being renamed
it along with its config and reflog will be copied to a new name.

With a -d or -D option, <branchname> will be deleted. You may specify more than one branch for deletion. If
the branch currently has a reflog then the reflog will also be deleted.

Use -r together with -d to delete remote-tracking branches. Note, that it only makes sense to delete
remote-tracking branches if they no longer exist in the remote repository or if git fetch was configured not
to fetch them again. See also the prune subcommand of git-remote(1) for a way to clean up all obsolete
remote-tracking branches.
")]
pub(crate) struct BranchArgs {
    /// The name of the branch to create, delete, or rename. The new branch name must pass all checks defined by git-check-
    /// ref-format(1). Some of these checks may restrict the characters allowed in a branch name.
    #[clap(value_name("branchname"))]
    pub branch_name: Option<String>,

    /// The new name for a renamed branch
    #[clap(value_name("newbranch"))]
    pub new_branch: Option<String>,

    /// Delete a branch. The branch must be fully merged in its upstream branch, or in HEAD if no upstream was set
    /// with --track or --set-upstream-to.
    #[arg(short, long)]
    pub delete: bool,

    /// Move/rename a branch and the corresponding reflog.
    #[arg(short, long("move"))]
    pub move_flag: bool,
}
