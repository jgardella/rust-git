use std::{fs::{DirBuilder, self, File}, path::{PathBuf, Path}, io::Write};
use crate::{RustGitError, config::{GitConfig, CoreConfig, ExtensionsConfig}};

use super::cli::{InitArgs, InitPermissionFlag, HashAlgorithm};

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

/// Creates basic git repository folder structure and default files:
/// .git
///     - HEAD (current repo head)
///     - config (repo-level config)
///     - description (repo description)
///     - objects
///         - info
///         - pack
///     - info
///         - exclude
///     - hooks (directory containing git hooks, will keep it empty until hooks are implemented)
///     - refs
///         - heads
///         - tags
fn create_git_repo(cmd: &InitCommand, project_dir: &PathBuf, git_dir: &PathBuf) -> Result<PathBuf, RustGitError>
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

    let head_file_path = git_repo_dir.join("HEAD");
    let mut head_file = File::create(head_file_path)?;

    // Real Git has some validation on the ref format, omitted for now:
    // https://github.com/git/git/blob/master/setup.c#L1952

    // There's also a more complex implementation of working with the HEAD file,
    // for now we'll keep it simple and re-visit when we start doing actual operations on it.
    // https://github.com/git/git/blob/master/refs/reftable-backend.c#L2230
    let initial_branch_ref = format!("ref: refs/heads/{}", &cmd.args.initial_branch);

    head_file.write_all(initial_branch_ref.as_bytes())?;

    let config = init_config(cmd, &git_repo_dir);
    let config_file_path = git_repo_dir.join("config");
    let mut config_file = File::create(config_file_path)?;
    // TODO: using TOML for ease of use, but the git config format isn't TOML
    // Might need to implement a custom serde for that format.
    config_file.write_all(toml::to_string_pretty(&config)?.as_bytes())?;

    Ok(fs::canonicalize(git_repo_dir)?)
}

fn init_config(cmd: &InitCommand, git_repo_dir: &PathBuf) -> GitConfig {
	// Note that we initialize the repository version to 1 when the ref
	// storage format is unknown. This is on purpose so that we can add the
	// correct object format to the config during git-clone(1). The format
	// version will get adjusted by git-clone(1) once it has learned about
	// the remote repository's format.
    let repo_version =
        if cmd.args.object_format != HashAlgorithm::Sha1 {
            0
        } else {
            1
        };

    let object_format = 
        if cmd.args.object_format != HashAlgorithm::Sha1 {
            Some(cmd.args.object_format.to_string())
        } else {
            None
        };
    
    // Omitted a few things in config initialization:
    // - setting work-tree: https://github.com/git/git/blob/master/setup.c#L1871C1-L1883C2
    // - checking if symlink is supported in work tree: https://github.com/git/git/blob/master/setup.c#L2044-L2053

    let ignorecase =
        // The HEAD file should already be created by this point,
        // so we can test if the filesystem is case-sensitive this way.
        if git_repo_dir.join("head").exists() {
            Some(true)
        } else {
            None
        };

    GitConfig { 
        core: Some(CoreConfig {
            repositoryformatversion: Some(repo_version),
            bare: Some(cmd.args.bare),
            ignorecase: ignorecase,
            ..Default::default()
        }),
        extensions: Some(ExtensionsConfig {
            objectformat: object_format,
            ..Default::default()
        }),
        ..Default::default()
    }
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

    create_git_repo(&cmd, &root_dir, &git_dir)
}
