use clap::Args;

#[derive(Args, Debug)]
#[command(about = "Show information about files in the index and the working tree")]
#[command(long_about = "This merges the file listing in the directory cache index with the actual working directory list, and shows different combinations of the two.")]
pub(crate) struct LsFilesArgs {
    /// Show cached files in the output (default)
    #[arg(short, long)]
    pub cached: bool,

    /// Show staged contents' mode bits, object name and stage number in the output.
    #[arg(short, long)]
    pub stage: bool,

    /// After each line that describes a file, add more data about its cache entry. This is intended to show as much information as possible for manual
    /// inspection; the exact format may change at any time.
    #[arg(long)]
    pub debug: bool,
}