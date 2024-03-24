use std::{fs::{DirBuilder, self, File}, path::{PathBuf, Path}, io::Write};
use crate::{command::GitCommand, config::{CoreConfig, ExtensionsConfig, GitConfig}, repo::GitRepo, RustGitError};

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
    config.write(&git_repo_dir)?;

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

    // Omitted a few things in config initialization:
    // - setting work-tree: https://github.com/git/git/blob/master/setup.c#L1871C1-L1883C2
    // - checking if symlink is supported in work tree: https://github.com/git/git/blob/master/setup.c#L2044-L2053

    let ignorecase =
        // The HEAD file should already be created by this point,
        // so we can test if the filesystem is case-sensitive this way.
        git_repo_dir.join("head").exists();

    GitConfig { 
        core: CoreConfig {
            repositoryformatversion: repo_version,
            bare: cmd.args.bare,
            ignorecase: ignorecase,
            ..Default::default()
        },
        extensions: ExtensionsConfig {
            objectformat: cmd.args.object_format,
            ..Default::default()
        },
        ..Default::default()
    }
}



impl GitCommand for InitCommand {

    fn execute(&self, _: GitRepo) -> Result<(), RustGitError>
    {
        // Create base directory, if specified.
        let root_dir =
            match &self.args.directory {
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
        if let Some(_) = self.args.separate_git_dir {
            return Err(RustGitError::new(String::from("--separate-git-dir not supported")));
        }

        if let Some(_) = self.args.template {
            return Err(RustGitError::new(String::from("--template not supported")));
        }

        if self.args.shared != InitPermissionFlag::Group {
            return Err(RustGitError::new(String::from("--shared not supported")));
        }

        if (self.git_dir.is_none() || self.args.bare) && self.git_work_tree.is_some() {
            return Err(RustGitError::new(String::from("work-tree can't be set without specifying git-dir")));
        }

        if self.args.bare && self.args.separate_git_dir.is_some() {
            return Err(RustGitError::new(String::from("--separate-git-dir incompatible with bare repository")));
        }

        let default_git_dir = &PathBuf::from(DEFAULT_GIT_DIR);
        let git_dir = self.git_dir.as_ref().unwrap_or(default_git_dir);

        let git_repo_dir = create_git_repo(&self, &root_dir, &git_dir)?;
        let git_dir_display = git_repo_dir.display();
        println!("Initialized empty Git repository in {git_dir_display}");

        Ok(())
    }
}
