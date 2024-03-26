use std::path::PathBuf;

use clap::Args;

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum ChmodFlag {
    Executable,
    NonExecutable,
}

fn parse_chmod_flag(s: &str) -> Result<ChmodFlag, String> {
    match s {
        "+x" => Ok(ChmodFlag::Executable),
        "-x" => Ok(ChmodFlag::NonExecutable),
        other => {
            Err(String::from(format!("-chmod param '{other}' must be either -x or +x")))
        }
    }
}


#[derive(Args, Debug)]
#[command(about = "Add file contents to the index")]
#[command(long_about = "
This command updates the index using the current content found in the working tree, to prepare the content staged for the next
commit. It typically adds the current content of existing paths as a whole, but with some options it can also be used to add
content with only part of the changes made to the working tree files applied, or remove paths that do not exist in the working tree
anymore.

The \"index\" holds a snapshot of the content of the working tree, and it is this snapshot that is taken as the contents of the next
commit. Thus after making any changes to the working tree, and before running the commit command, you must use the add command to
add any new or modified files to the index.

This command can be performed multiple times before a commit. It only adds the content of the specified file(s) at the time the add
command is run; if you want subsequent changes included in the next commit, then you must run git add again to add the new content
to the index.

The git status command can be used to obtain a summary of which files have changes that are staged for the next commit.

The git add command will not add ignored files by default. If any ignored files were explicitly specified on the command line, git
add will fail with a list of ignored files. Ignored files reached by directory recursion or filename globbing performed by Git
(quote your globs before the shell) will be silently ignored. The git add command can be used to add ignored files with the -f
(force) option.

Please see git-commit(1) for alternative ways to add content to a commit.
")]
pub(crate) struct AddArgs {

    /// Files to add content from. Fileglobs (e.g.  *.c) can be given to add all matching files. Also a leading directory name (e.g.
    /// dir to add dir/file1 and dir/file2) can be given to update the index to match the current state of the directory as a whole
    /// (e.g. specifying dir will record not just a file dir/file1 modified in the working tree, a file dir/file2 added to the working
    /// tree, but also a file dir/file3 removed from the working tree). Note that older versions of Git used to ignore removed files;
    /// use --no-all option if you want to add modified or new files but ignore removed ones.
    /// 
    /// For more details about the <pathspec> syntax, see the pathspec entry in gitglossary(7).
    pub pathspec: Option<String>,

    /// Don’t actually add the file(s), just show if they exist and/or will be ignored.
    #[arg(long, short('n'), conflicts_with="interactive", conflicts_with="patch", conflicts_with="pathspec_from_file")]
    pub dry_run: bool,

    /// Be verbose.
    #[arg(long, short)]
    pub verbose: bool,

    /// Allow adding otherwise ignored files.
    #[arg(long, short)]
    pub force: bool,

    /// Allow updating index entries outside of the sparse-checkout cone. Normally, git add refuses to update index entries whose paths
    /// do not fit within the sparse-checkout cone, since those files might be removed from the working tree without warning. See git-
    /// sparse-checkout(1) for more details.
    #[arg(long)]
    pub sparse: bool,

    /// Add modified contents in the working tree interactively to the index. Optional path arguments may be supplied to limit
    /// operation to a subset of the working tree. See “Interactive mode” for details.
    #[arg(long, short)]
    pub interactive: bool,

    /// Interactively choose hunks of patch between the index and the work tree and add them to the index. This gives the user a chance
    /// to review the difference before adding modified contents to the index.
    /// 
    /// This effectively runs add --interactive, but bypasses the initial command menu and directly jumps to the patch subcommand. See
    /// “Interactive mode” for details.
    #[arg(long, short)]
    pub patch: bool, 

    /// Open the diff vs. the index in an editor and let the user edit it. After the editor was closed, adjust the hunk headers and
    /// apply the patch to the index.
    /// 
    /// The intent of this option is to pick and choose lines of the patch to apply, or even to modify the contents of lines to be
    /// staged. This can be quicker and more flexible than using the interactive hunk selector. However, it is easy to confuse oneself
    /// and create a patch that does not apply to the index. See EDITING PATCHES below.
    #[arg(long, short, conflicts_with="pathspec_from_file")]
    pub edit: bool, 

    /// Update the index just where it already has an entry matching <pathspec>. This removes as well as modifies index entries to
    /// match the working tree, but adds no new files.
    /// 
    /// If no <pathspec> is given when -u option is used, all tracked files in the entire working tree are updated (old versions of Git
    /// used to limit the update to the current directory and its subdirectories).
    #[arg(long, short)]
    pub update: bool, 

    /// Update the index not only where the working tree has a file matching <pathspec> but also where the index already has an entry.
    /// This adds, modifies, and removes index entries to match the working tree.
    /// 
    /// If no <pathspec> is given when -A option is used, all files in the entire working tree are updated (old versions of Git used to
    /// limit the update to the current directory and its subdirectories).
    #[arg(long, short='A', alias="all", conflicts_with="update")]
    pub no_ignore_removal: bool, 

    /// Update the index by adding new files that are unknown to the index and files modified in the working tree, but ignore files
    /// that have been removed from the working tree. This option is a no-op when no <pathspec> is used.
    /// 
    /// This option is primarily to help users who are used to older versions of Git, whose "git add <pathspec>..." was a synonym for
    /// "git add --no-all <pathspec>...", i.e. ignored removed files.
    #[arg(long, alias="no-all")]
    pub ignore_removal: bool, 

    /// Record only the fact that the path will be added later. An entry for the path is placed in the index with no content. This is
    /// useful for, among other things, showing the unstaged content of such files with git diff and committing them with git commit
    /// -a.
    #[arg(long, short='N')]
    pub intent_to_add: bool,

    /// Don’t add the file(s), but only refresh their stat() information in the index.
    #[arg(long)]
    pub refresh: bool,

    /// If some files could not be added because of errors indexing them, do not abort the operation, but continue adding the others.
    /// The command shall still exit with non-zero status. The configuration variable add.ignoreErrors can be set to true to make this
    /// the default behaviour.
    #[arg(long)]
    pub ignore_errors: bool,

    /// This option can only be used together with --dry-run. By using this option the user can check if any of the given files would
    /// be ignored, no matter if they are already present in the work tree or not.
    #[arg(long, requires="dry_run")]
    pub ignore_missing: bool,

    /// By default, git add will warn when adding an embedded repository to the index without using git submodule add to create an
    /// entry in .gitmodules. This option will suppress the warning (e.g., if you are manually performing operations on submodules).
    #[arg(long)]
    pub no_warn_embedded_repo: bool,

    /// Apply the "clean" process freshly to all tracked files to forcibly add them again to the index. This is useful after changing
    /// core.autocrlf configuration or the text attribute in order to correct files added with wrong CRLF/LF line endings. This option
    /// implies -u. Lone CR characters are untouched, thus while a CRLF cleans to LF, a CRCRLF sequence is only partially cleaned to
    /// CRLF.
    #[arg(long)]
    pub renormalize: bool,

    /// Override the executable bit of the added files. The executable bit is only changed in the index, the files on disk are left
    /// unchanged.
    #[arg(long, value_parser=parse_chmod_flag)]
    pub chmod: Option<ChmodFlag>,

    /// Pathspec is passed in <file> instead of commandline args. If <file> is exactly - then standard input is used. Pathspec elements
    /// are separated by LF or CR/LF. Pathspec elements can be quoted as explained for the configuration variable core.quotePath (see
    /// git-config(1)). See also --pathspec-file-nul and global --literal-pathspecs.
    #[arg(long, value_name="file", conflicts_with="pathspec")]
    pub pathspec_from_file: Option<PathBuf>,

    /// Only meaningful with --pathspec-from-file. Pathspec elements are separated with NUL character and all other characters are
    /// taken literally (including newlines and quotes).
    #[arg(long, requires="pathspec_from_file")]
    pub pathspec_file_nul: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_chmod_flag()
    {
        assert_eq!(parse_chmod_flag("+x"), Ok(ChmodFlag::Executable));
        assert_eq!(parse_chmod_flag("-x"), Ok(ChmodFlag::NonExecutable));
        assert_eq!(parse_chmod_flag("invalid"), Err(String::from("-chmod param 'invalid' must be either -x or +x")));
    }
}
