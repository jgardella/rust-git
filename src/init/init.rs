use std::{fs::{DirBuilder, self}, path::{PathBuf, Path}};
use crate::RustGitError;

use super::cli::{InitArgs, InitPermissionFlag};

const DEFAULT_GIT_DIR: &str = ".git";

pub(crate) struct InitCommand {
    args: InitArgs,

    git_dir: Option<PathBuf>,

    git_work_tree: Option<PathBuf>
}

impl InitCommand {
    pub fn new(args: InitArgs, git_dir: Option<PathBuf>, git_work_tree: Option<PathBuf>) -> InitCommand {
        InitCommand {
            args,
            git_dir,
            git_work_tree,
        }
    }
}

/// Creates basic git repository folder structure:
/// .git
///     - objects
///         - info
///         - pack
///     - info
///     - hooks
///     - refs
///         - heads
///         - tags
fn create_dirs(cmd: &InitCommand, project_dir: &PathBuf, git_dir: &PathBuf) -> Result<PathBuf, RustGitError>
{
    let mut git_repo_dir = project_dir.to_path_buf();

    let dir_builder = DirBuilder::new();

    if !cmd.args.bare {
        git_repo_dir.push(git_dir);

        if git_repo_dir.exists() {
            return Err(RustGitError::new(format!("{git_repo_dir:#?} already exists")))
        }

        dir_builder.create(&git_repo_dir)?;
    }

    let objects_dir = git_repo_dir.join("objects");
    let objects_info_dir = objects_dir.join("info");
    let objects_pack_dir = objects_dir.join("pack");
    let info_dir = git_repo_dir.join("info");
    let hooks_dir = git_repo_dir.join("hooks");
    let refs_dir = git_repo_dir.join("refs");
    let refs_heads_dir = refs_dir.join("heads");
    let refs_tags_dir = refs_dir.join("tags");

    dir_builder.create(objects_dir)?;
    dir_builder.create(objects_info_dir)?;
    dir_builder.create(objects_pack_dir)?;
    dir_builder.create(info_dir)?;
    dir_builder.create(hooks_dir)?;
    dir_builder.create(refs_dir)?;
    dir_builder.create(refs_heads_dir)?;
    dir_builder.create(refs_tags_dir)?;
        
    Ok(fs::canonicalize(git_repo_dir)?)
}


pub(crate) fn init_repository(cmd: &InitCommand) -> Result<PathBuf, RustGitError>
{
    // Create base directory, if specified.
    let root_dir =
        match &cmd.args.directory {
            Some(directory) => {
                // Creation of the directory is also more complex, but we keep it simple.
                // Real git does something special with "creating leading directories":
                // https://github.com/git/git/blob/master/builtin/init-db.c#L133
                let mut dir_builder = DirBuilder::new();
                dir_builder.recursive(true);

                dir_builder.create(directory)
                .map_err(|_| RustGitError::new(format!("Failed to create directory: {directory}")))?;

                Path::new(&directory).to_path_buf()
            }
            None => Path::new("./").to_path_buf()
        };

    // Real Git has some logic to guess if the repository is bare or not, we omit that for simplicity:
    // https://github.com/git/git/blob/master/builtin/init-db.c#L218-L219

    // Omitting implementaion of --separate-git-dir, --template-dir, and --shared for now, for simplicity
    if let Some(_) = cmd.args.separate_git_dir {
        return Err(RustGitError::new(String::from("--separate-git-dir not supported")));
    }

    if let Some(_) = cmd.args.template {
        return Err(RustGitError::new(String::from("--template not supported")));
    }

    if cmd.args.shared != InitPermissionFlag::Group {
        return Err(RustGitError::new(String::from("--shared not supported")));
    }

    if (cmd.git_dir.is_none() || cmd.args.bare) && cmd.git_work_tree.is_some() {
        return Err(RustGitError::new(String::from("work-tree can't be set without specifying git-dir")));
    }

    if cmd.args.bare && cmd.args.separate_git_dir.is_some() {
        return Err(RustGitError::new(String::from("--separate-git-dir incompatible with bare repository")));
    }

    let default_git_dir = &PathBuf::from(DEFAULT_GIT_DIR);
    let git_dir = cmd.git_dir.as_ref().unwrap_or(default_git_dir);

    create_dirs(&cmd, &root_dir, &git_dir)
}
