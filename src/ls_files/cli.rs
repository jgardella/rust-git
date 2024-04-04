use clap::Args;

#[derive(Args, Debug)]
#[command(about = "Show information about files in the index and the working tree")]
#[command(long_about = "This merges the file listing in the directory cache index with the actual working directory list, and shows different combinations of the two.")]
pub(crate) struct LsFilesArgs {
    /// Show cached files in the output (default)
    #[arg(short, long)]
    cached: bool,

    /// Show deleted files in the output
    #[arg(short, long)]
    deleted: bool,

    /// Show modified files in the output
    #[arg(short, long)]
    modified: bool,

    /// Show other (i.e. untracked) files in the output
    #[arg(short, long)]
    others: bool,

    /// Show only ignored files in the output. When showing files in the index, print only those matched by an exclude pattern. When showing "other"
    /// files, show only those matched by an exclude pattern. Standard ignore rules are not automatically activated, therefore at least one of the
    /// --exclude* options is required.
    #[arg(short, long)]
    ignored: bool,

    /// Show staged contents' mode bits, object name and stage number in the output.
    #[arg(short, long)]
    stage: bool,

    /// After each line that describes a file, add more data about its cache entry. This is intended to show as much information as possible for manual
    /// inspection; the exact format may change at any time.
    #[arg(long)]
    debug: bool,
}