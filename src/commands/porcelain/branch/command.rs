use crate::{command::GitCommand, repo::RepoState, RustGitError};

use super::cli::BranchArgs;

pub(crate) struct CreateBranch {
    branch_name: String,
}

pub(crate) struct RenameBranch {
    new_branch_name: String,
}

pub(crate) struct DeleteBranch {
    branch_name: String,
}

pub(crate) enum BranchCommand {
    ListBranches(),
    CreateBranch(CreateBranch),
    RenameBranch(RenameBranch),
    DeleteBranch(DeleteBranch),
}

impl BranchCommand {
    pub fn new(args: BranchArgs) -> Result<BranchCommand, RustGitError> {
        if args.delete {
            if let Some(branch_name) = args.branch_name {
                return Ok(BranchCommand::DeleteBranch(DeleteBranch { branch_name }));
            } else {
                return Err(RustGitError::new("expected branch name for deletion"));
            }
        }

        if args.move_flag {
            if let Some(branch_name) = args.branch_name {
                return Ok(BranchCommand::RenameBranch(RenameBranch {
                    new_branch_name: branch_name,
                }));
            } else {
                return Err(RustGitError::new("expected branch name for move"));
            }
        }

        if let Some(branch_name) = args.branch_name {
            return Ok(BranchCommand::CreateBranch(CreateBranch { branch_name }));
        } else {
            return Ok(BranchCommand::ListBranches());
        }
    }
}

impl GitCommand for BranchCommand {
    fn execute(&self, repo_state: RepoState) -> Result<(), RustGitError> {
        let repo = repo_state.try_get()?;

        match self {
            BranchCommand::CreateBranch(create_cmd) => {
                if let (_, Some(head_ref)) = repo.refs.get_head_ref()? {
                    return repo.refs.create_ref(&create_cmd.branch_name, &head_ref);
                } else {
                    return Err(RustGitError::new("failed to load HEAD"));
                }
            }
            BranchCommand::RenameBranch(rename_cmd) => {
                if let (Some(old_branch_name), _) = repo.refs.get_head_ref()? {
                    return repo
                        .refs
                        .rename_ref(&old_branch_name, &rename_cmd.new_branch_name);
                } else {
                    return Err(RustGitError::new("failed to load HEAD"));
                }
            }
            BranchCommand::ListBranches() => {
                let refs = repo.refs.list_refs()?;
                for ref_value in refs {
                    println!("{}", ref_value)
                }
            }
            BranchCommand::DeleteBranch(delete_cmd) => {
                repo.refs.delete_ref(&delete_cmd.branch_name)?
            }
        }

        Ok(())
    }
}
