use clap::Args;

#[derive(Args, Debug)]
#[command(about = "Remove files from the working tree and from the index")]
#[command(long_about = "
Remove files from the index, or from the working tree and the index. git rm will not remove a file from just your working directory. (There is no
option to remove a file only from the working tree and yet keep it in the index; use /bin/rm if you want to do that.) The files being removed have to
be identical to the tip of the branch, and no updates to their contents can be staged in the index, though that default behavior can be overridden
with the -f option. When --cached is given, the staged content has to match either the tip of the branch or the file on disk, allowing the file to be
removed from just the index.")]
pub(crate) struct RmArgs {

    /// Files to remove. Fileglobs (e.g.  *.c) can be given to remove all matching files. If you want Git to expand file glob characters, you may need to
    /// shell-escape them. A leading directory name (e.g.  dir to remove dir/file1 and dir/file2) can be given to remove all files in the directory, and
    /// recursively all sub-directories, but this requires the -r option to be explicitly given.
    files: Vec<String>,

    /// Override the up-to-date check.
    #[arg(short, long)]
    force: bool,

    /// Don’t actually remove any file(s). Instead, just show if they exist in the index and would otherwise be removed by the command.
    #[arg(short('n'), long)]
    dry_run: bool,

    /// Allow recursive removal when a leading directory name is given.
    #[arg(short)]
    recursive: bool,

    /// Use this option to unstage and remove paths only from the index. Working tree files, whether modified or not, will be left alone.
    #[arg(long)]
    cached: bool,

    /// Exit with a zero status even if no files matched.
    #[arg(long)]
    ignore_unmatch: bool,

    /// git rm normally outputs one line (in the form of an rm command) for each file removed. This option suppresses that output.
    #[arg(short, long)]
    quiet: bool,
}
