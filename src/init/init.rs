use std::{fs::{DirBuilder, self}, path::{PathBuf, Path}};
use crate::RustGitError;

use super::cli::InitArgs;

const DEFAULT_GIT_DIR: &str = ".git";

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



pub(crate) fn init_repository(args: &InitArgs) -> Result<PathBuf, RustGitError> // Not sure yet what we should return. Git returns an int.
{
    // Real implementation this this is much more involved, but let's keep it simple for now.
    // https://github.com/git/git/blob/master/builtin/init-db.c#L112-L118
    let real_git_dir = args.separate_git_dir.as_ref().map(fs::canonicalize);
    let template_dir = args.separate_git_dir.as_ref().map(fs::canonicalize);

    let root_dir =
        match &args.directory {
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


    if let Some(object_format) = &args.object_format {

    }

    create_dirs(&root_dir, args.bare)
}
