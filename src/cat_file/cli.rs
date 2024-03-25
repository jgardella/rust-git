use std::path::PathBuf;

use clap::Args;

use crate::repo::ObjectType;

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

}
