
use std::fs;

use crate::{command::GitCommand, repo::RepoState, RustGitError};

use super::cli::RmArgs;

pub(crate) struct RmCommand {
    args: RmArgs,

    // TODO: add base args
}

impl RmCommand {
    pub fn new(args: RmArgs) -> RmCommand {
        RmCommand {
            args
        }
    }
}

impl GitCommand for RmCommand {
    fn execute(&self, repo_state: RepoState) -> Result<(), RustGitError>
    {
        if self.args.files.is_empty() {
            return Err(RustGitError::new("No pathspec was given. Which files should I remove?"));
        }

        let mut repo = repo_state.try_get()?;

        // TODO: refresh index
        // https://github.com/git/git/blob/master/read-cache.c#L1527

        // If not forced, the file, the index and the HEAD (if exists)
        // must match; but the file can already been removed, since
        // this sequence is a natural "novice" way:
        // 
        //   rm F; git rm F
        //  
        // Further, if HEAD commit exists, "diff-index --cached" must
        // report no changes unless forced.
        if !self.args.force {
            // TODO: add check for local modifications here, after adding support for commits
            // https://github.com/git/git/blob/master/builtin/rm.c#L99
            ()
        }

        // First remove the names from the index: we won't commit
        // the index unless all of them succeed.
        let paths_to_remove = repo.index.filter_entries(|entry| {
            if self.args.files.contains(&entry.path_name) {
                if !self.args.quiet {
                    println!("rm '{}'", entry.path_name);
                }

                return false;
            }

            return true;
        });

        if paths_to_remove.is_empty() {
            if self.args.ignore_unmatch {
                return Ok(());
            } else {
                return Err(RustGitError::new("No files matched for rm"));
            }
        }

        if self.args.dry_run {
            return Ok(());
        }

        // Then, unless we used "--cached", remove the filenames from
        // the workspace. If we fail to remove the first one, we
        // abort the "git rm" (but once we've successfully removed
        // any file at all, we'll go ahead and commit to it all:
        // by then we've already committed ourselves and can't fail
        // in the middle)
        if !self.args.cached {
            for (i, path) in paths_to_remove.iter().enumerate() {
                match (i, fs::remove_file(path)) {
                    (0, Err(_)) => return Err(RustGitError::new(format!("git rm: '{path}'"))),
                    _ => ()
                }
            }
        }

        repo.write_index()?;

        Ok(())
    }
}
