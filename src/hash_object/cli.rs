use std::path::PathBuf;

use clap::Args;

use crate::object::GitObjectType;

#[derive(Args, Debug)]
#[command(about = "Compute object ID and optionally creates a blob from a file")]
#[command(long_about = "
Computes the object ID value for an object with specified type with the contents of the named file (which can be outside of the
work tree), and optionally writes the resulting object into the object database. Reports its object ID to its standard output. When
<type> is not specified, it defaults to \"blob\".
")]
pub(crate) struct HashObjectArgs {
    /// Specify the type (default: "blob").
    #[arg(short='t', default_value="blob", value_name="type")]
    pub object_type: GitObjectType,

    /// Actually write the object into the object database.
    #[arg(short)]
    pub write: bool,

    /// Read the object from standard input instead of from a file.
    #[arg(long)]
    pub stdin: bool,

    /// Read file names from the standard input, one per line, instead of from the command-line.
    #[arg(long, conflicts_with="stdin", conflicts_with="path", conflicts_with="files")]
    pub stdin_paths: bool,

    /// Hash object as it were located at the given path. The location of file does not directly influence on the hash value, but path
    /// is used to determine what Git filters should be applied to the object before it can be placed to the object database, and, as
    /// result of applying filters, the actual blob put into the object database may differ from the given file. This option is mainly
    /// useful for hashing temporary files located outside of the working directory or files read from stdin.
    #[arg(long, conflicts_with="no_filters")]
    pub path: Option<PathBuf>,

    /// Hash the contents as is, ignoring any input filter that would have been chosen by the attributes mechanism, including the
    /// end-of-line conversion. If the file is read from standard input then this is always implied, unless the --path option is given.
    #[arg(long)]
    pub no_filters: bool,

    /// Allow --stdin to hash any garbage into a loose object which might not otherwise pass standard object parsing or git-fsck
    /// checks. Useful for stress-testing Git itself or reproducing characteristics of corrupt or bogus objects encountered in the
    /// wild.
    #[arg(long)]
    pub literally: bool,

    pub files: Vec<String>,
}
