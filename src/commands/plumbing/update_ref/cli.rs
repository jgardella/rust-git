use std::path::PathBuf;

use clap::Args;

#[derive(Args, Debug)]
#[command(about = "Update the object name stored in a ref safely")]
#[command(long_about = "
Given two arguments, stores the <newvalue> in the <ref>, possibly dereferencing the symbolic refs. E.g. git
update-ref HEAD <newvalue> updates the current branch head to the new object.

Given three arguments, stores the <newvalue> in the <ref>, possibly dereferencing the symbolic refs, after
verifying that the current value of the <ref> matches <oldvalue>. E.g. git update-ref refs/heads/master
<newvalue> <oldvalue> updates the master branch head to <newvalue> only if its current value is <oldvalue>.
You can specify 40 \"0\" or an empty string as <oldvalue> to make sure that the ref you are creating does not
exist.

It also allows a \"ref\" file to be a symbolic pointer to another ref file by starting with the four-byte header
sequence of \"ref\".

More importantly, it allows the update of a ref file to follow these symbolic pointers, whether they are
symlinks or these \"regular file symbolic refs\". It follows real symlinks only if they start with \"refs/\":
otherwise it will just try to read them and update them as a regular file (i.e. it will allow the filesystem
to follow them, but will overwrite such a symlink to somewhere else with a regular filename).

If --no-deref is given, <ref> itself is overwritten, rather than the result of following the symbolic
pointers.")]
pub(crate) struct UpdateRefArgs {
    #[clap(value_name("ref"))]
    pub git_ref: String,

    #[clap(value_name("newvalue"))]
    pub new_value: String,

    #[clap(value_name("oldvalue"))]
    pub old_value: Option<String>,
}
