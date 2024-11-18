use std::path::PathBuf;

use clap::Args;

#[derive(Args, Debug)]
#[command(about = "Create a new commit object")]
#[command(long_about = "
This is usually not what an end user wants to run directly. See git-commit(1) instead.

Creates a new commit object based on the provided tree object and emits the new commit object id on stdout.
The log message is read from the standard input, unless -m or -F options are given.

The -m and -F options can be given any number of times, in any order. The commit log message will be composed
in the order in which the options are given.

A commit object may have any number of parents. With exactly one parent, it is an ordinary commit. Having more
than one parent makes the commit a merge between several lines of history. Initial (root) commits have no
parents.

While a tree represents a particular directory state of a working directory, a commit represents that state in
\"time\", and explains how to get there.

Normally a commit would identify a new \"HEAD\" state, and while Git doesn't care where you save the note about
that state, in practice we tend to just write the result to the file that is pointed at by .git/HEAD, so that
we can always see what the last committed state was.
")]
pub(crate) struct CommitTreeArgs {
    /// An existing tree object.
    pub tree: String,

    /// Each -p indicates the id of a parent commit object.
    #[arg(short)]
    pub parents: Vec<String>,

    /// A paragraph in the commit log message. This can be given more than once and each <message> becomes its own
    /// paragraph.
    #[arg(short)]
    pub messages: Vec<String>,
}
