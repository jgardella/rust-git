use std::{fs::{DirBuilder, self}, path::{PathBuf, Path}};
use crate::RustGitError;

use super::cli::InitArgs;

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

fn create_dirs(root_dir: &PathBuf, bare: bool) -> Result<PathBuf, RustGitError>
{
    let mut dirs_to_create = Vec::<&PathBuf>::new();

    let mut git_dir = root_dir.to_path_buf();

    if !bare {
        git_dir.push(DEFAULT_GIT_DIR);

        if git_dir.exists() {
            return Err(RustGitError::new(format!("{git_dir:#?} already exists")))
        }

        dirs_to_create.push(&git_dir);
    }

    let objects_dir = git_dir.join("objects");
    let objects_info_dir = objects_dir.join("info");
    let objects_pack_dir = objects_dir.join("pack");
    dirs_to_create.extend([&objects_dir, &objects_info_dir, &objects_pack_dir]);

    // TODO: set directory permission based on InitArgs.shared and other configs
    let dir_builder = DirBuilder::new();

    let dir_create_result: Result<Vec<()>, _> = 
        dirs_to_create
        .iter()
        .map(|path| dir_builder.create(path))
        .collect();

    return 
        dir_create_result
        .map(|_| fs::canonicalize(git_dir).unwrap())
        .map_err(|err| RustGitError::new(err.to_string()));
}


pub(crate) fn init_repository(cmd: &InitCommand) -> Result<PathBuf, RustGitError> // Not sure yet what we should return. Git returns an int.
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

    // Omitting implementaion of --separate-git-dir and --template-dir for now, for simplicity
    if let Some(_) = cmd.args.separate_git_dir {
        return Err(RustGitError::new(String::from("--separate-git-dir not supported")));
    }

    if let Some(_) = cmd.args.template {
        return Err(RustGitError::new(String::from("--template not supported")));
    }

    if (cmd.git_dir.is_none() || cmd.args.bare) && cmd.git_work_tree.is_some() {
        return Err(RustGitError::new(String::from("work-tree can't be set without specifying git-dir")));
    }

    if cmd.args.bare && cmd.args.separate_git_dir.is_some() {
        return Err(RustGitError::new(String::from("--separate-git-dir incompatible with bare repository")));
    }

    create_dirs(&root_dir, cmd.args.bare)
}
