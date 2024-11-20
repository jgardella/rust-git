use std::path::PathBuf;

use clap::Args;

#[derive(Args, Debug)]
#[command(about = "Read, modify, and delete symbolic refs")]
#[command(long_about = "
Given one argument, reads which branch head the given symbolic ref refers to and outputs its path, relative to
the .git/ directory. Typically you would give HEAD as the <name> argument to see which branch your working
tree is on.

Given two arguments, creates or updates a symbolic ref <name> to point at the given branch <ref>.

Given --delete and an additional argument, deletes the given symbolic ref.

A symbolic ref is a regular file that stores a string that begins with ref: refs/. For example, your .git/HEAD
is a regular file whose contents is ref: refs/heads/master.
")]
pub(crate) struct SymbolicRefArgs {
    #[clap(value_name("name"))]
    pub ref_name: String,

    #[clap(value_name("ref"))]
    pub new_value: Option<String>,

    /// Delete the symbolic ref <name>.
    #[arg(short, long)]
    pub delete: bool,

    /// Do not issue an error message if the <name> is not a symbolic ref but a detached HEAD; instead exit with
    /// non-zero status silently.
    #[arg(short, long)]
    pub quiet: bool,

    /// When showing the value of <name> as a symbolic ref, try to shorten the value, e.g. from refs/heads/master
    /// to master.
    #[arg(long)]
    pub short: bool,
}
