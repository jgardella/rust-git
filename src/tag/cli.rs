use std::path::PathBuf;

use clap::Args;

#[derive(Args, Debug)]
#[command(about = "Create, list, delete or verify a tag object signed with GPG")]
#[command(long_about = "
Add a tag reference in refs/tags/, unless -d/-l/-v is given to delete, list or verify tags.

Unless -f is given, the named tag must not yet exist.

If one of -a, -s, or -u <keyid> is passed, the command creates a tag object, and requires a tag message.
Unless -m <msg> or -F <file> is given, an editor is started for the user to type in the tag message.

If -m <msg> or -F <file> is given and -a, -s, and -u <keyid> are absent, -a is implied.

Otherwise, a tag reference that points directly at the given object (i.e., a lightweight tag) is created.

A GnuPG signed tag object will be created when -s or -u <keyid> is used. When -u <keyid> is not used, the
committer identity for the current user is used to find the GnuPG key for signing. The configuration variable
gpg.program is used to specify custom GnuPG binary.

Tag objects (created with -a, -s, or -u) are called \"annotated\" tags; they contain a creation date, the tagger
name and e-mail, a tagging message, and an optional GnuPG signature. Whereas a \"lightweight\" tag is simply a
name for an object (usually a commit object).

Annotated tags are meant for release while lightweight tags are meant for private or temporary object labels.
For this reason, some git commands for naming objects (like git describe) will ignore lightweight tags by
default.
")]
pub(crate) struct TagArgs {
    /// The name of the tag to create, delete, or describe. The new tag name must pass all checks defined by git-
    /// check-ref-format(1). Some of these checks may restrict the characters allowed in a tag name.
    #[clap(value_name("tagname"))]
    pub tag_name: Option<String>,

    /// The object that the new tag will refer to, usually a commit. Defaults to HEAD.
    #[clap(value_name("object"))]
    pub object: Option<String>,

    /// Replace an existing tag with the given name (instead of failing)
    #[arg(short, long)]
    pub force: bool,

    /// Delete existing tags with the given names.
    #[arg(short, long)]
    pub delete: bool,

    /// Use the given tag message (instead of prompting). If multiple -m options are given, their values are
    /// concatenated as separate paragraphs. Implies -a if none of -a, -s, or -u <keyid> is given.
    #[arg(short, long, value_name = "msg")]
    pub message: Option<String>,
}
