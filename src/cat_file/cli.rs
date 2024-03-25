use clap::Args;

use crate::repo::ObjectType;

#[derive(Args, Debug)]
#[group(multiple=false, requires="object")]
pub(crate) struct CatFileMode {
    /// Instead of the content, show the object type identified by <object>.
    #[arg(short('t'))]
    pub(crate) show_type: bool,

    /// Instead of the content, show the object size identified by <object>.
    #[arg(short('s'))]
    pub(crate) show_size: bool,

    /// Exit with zero status if <object> exists and is a valid object. If <object> is of an invalid format exit with
    /// non-zero and emits an error on stderr.
    #[arg(short('e'))]
    pub(crate) check: bool,

    /// Pretty-print the contents of <object> based on its type.
    #[arg(short)]
    pub(crate) print: bool,

}

#[derive(Args, Debug)]
#[command(about = "Provide content or type and size information for repository objects")]
#[command(long_about = "
In its first form, the command provides the content or the type of an object in the repository. The type is
required unless -t or -p is used to find the object type, or -s is used to find the object size, or --textconv or
--filters is used (which imply type \"blob\").

In the second form, a list of objects (separated by linefeeds) is provided on stdin, and the SHA-1, type, and size
of each object is printed on stdout. The output format can be overridden using the optional <format> argument. If
either --textconv or --filters was specified, the input is expected to list the object names followed by the path
name, separated by a single whitespace, so that the appropriate drivers can be determined.
")]
pub(crate) struct CatFileArgs {
    #[command(flatten)]
    pub(crate) mode: CatFileMode,

    // TODO: We are parsing the type and object as a vector; I couldn't find a better way to
    // represent the way C git handles the cat-file command using Clap.

    /// <type> 
    /// 
    /// Typically this matches the real type of <object> but asking for a type that can trivially be dereferenced from
    /// the given <object> is also permitted. An example is to ask for a "tree" with <object> being a commit object
    /// that contains it, or to ask for a "blob" with <object> being a tag object that points at it.
    /// 
    /// <object>
    /// 
    /// The name of the object to show. For a more complete list of ways to spell object names, see the "SPECIFYING
    /// REVISIONS" section in gitrevisions(7).
    #[arg(value_names=(["type", "object"]), num_args=0..3)]
    pub(crate) input: Vec<String>,
}
