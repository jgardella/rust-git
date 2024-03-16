use std::{fs::DirBuilder, path::Path};
use super::cli::InitArgs;

const DEFAULT_GIT_DIR: &str = ".git";

fn create_dirs(_: &InitArgs) -> Result<(), String>
{
    // TODO: handle config and args
    let git_dir = Path::new(DEFAULT_GIT_DIR);

    if git_dir.exists() {
        return Err(format!("{git_dir:#?} already exists"))
    }

    let objects_dir = git_dir.join("objects");
    let objects_info_dir = objects_dir.join("info");
    let objects_pack_dir = objects_dir.join("pack");

    let dirs_to_create = vec![
        git_dir,
        objects_dir.as_path(),
        objects_info_dir.as_path(),
        objects_pack_dir.as_path()
    ];

    // TODO: set directory permission based on InitArgs.shared and other configs
    let dir_builder = DirBuilder::new();

    let dir_create_result: Result<Vec<()>, _> = 
        dirs_to_create
        .iter()
        .map(|path| dir_builder.create(path))
        .collect();

    return 
        dir_create_result
        .map(|_| ())
        .map_err(|err| err.to_string());
}


pub(crate) fn init_repository(args: &InitArgs) -> Result<(), String> // Not sure yet what we should return. Git returns an int.
{
    create_dirs(args)
}
