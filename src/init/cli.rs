use std::{fmt::Display, path::PathBuf};

use clap::{Args, ValueEnum};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Default, Debug, Serialize, Deserialize, PartialEq, ValueEnum)]
#[serde(rename_all = "lowercase")]
pub(crate) enum HashAlgorithm {
    #[default]
    Sha1,
    Sha256
}

impl Display for HashAlgorithm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HashAlgorithm::Sha1 => write!(f, "sha1"),
            HashAlgorithm::Sha256 => write!(f, "sha256"),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum InitPermissionFlag {
    Umask,
    Group,
    All,
    Perm(u8)
}

fn parse_init_permission(s: &str) -> Result<InitPermissionFlag, String> {
    match s {
        "umask" | "false" => Ok(InitPermissionFlag::Umask),
        "group" | "true" => Ok(InitPermissionFlag::Group),
        "all" | "world" | "everybody" => Ok(InitPermissionFlag::All),
        perm => {
            if perm.len() == 4 && perm.chars().nth(0).unwrap() == '0' {
                u8::from_str_radix(&perm[1..], 8)
                .map(InitPermissionFlag::Perm)
                .map_err(|_| String::from("Invalid permission provided"))
            } else {
                Err(String::from("Invalid permission provided"))
            }
        }
    }
}

#[derive(Args, Debug)]
#[command(about = "Create an empty Git repository or reinitialize an existing one")]
#[command(long_about = "
This command creates an empty Git repository - basically a .git directory with subdirectories for objects, refs/heads, refs/tags, and template files. An initial branch without any commits
will be created (see the --initial-branch option below for its name).

If the $GIT_DIR environment variable is set then it specifies a path to use instead of ./.git for the base of the repository.

If the object storage directory is specified via the $GIT_OBJECT_DIRECTORY environment variable then the sha1 directories are created underneath - otherwise the default $GIT_DIR/objects
directory is used.

Running git init in an existing repository is safe. It will not overwrite things that are already there. The primary reason for rerunning git init is to pick up newly added templates (or to
move the repository to another place if --separate-git-dir is given).")]
pub(crate) struct InitArgs {
    /// Only print error and warning messages; all other output will be suppressed.
    #[arg(short, long)]
    pub quiet: bool,

    /// Create a bare repository. If GIT_DIR environment is not set, it is set to the current working directory.
    #[arg(long, conflicts_with="separate_git_dir")]
    pub bare: bool,

    /// Specify the directory from which templates will be used. (See the "TEMPLATE DIRECTORY" section below.)
    #[arg(long, value_name="template-directory")]
    pub template: Option<PathBuf>,

    /// Instead of initializing the repository as a directory to either $GIT_DIR or ./.git/, create a text file there containing the path to the actual repository. This file acts as
    /// filesystem-agnostic Git symbolic link to the repository.
    ///
    /// If this is reinitialization, the repository will be moved to the specified path.
    #[arg(long, value_name="git-dir")]
    pub separate_git_dir: Option<PathBuf>,

    /// Specify the given object format (hash algorithm) for the repository. The valid values are sha1 and (if enabled) sha256. sha1 is the default.
    /// 
    /// THIS OPTION IS EXPERIMENTAL! SHA-256 support is experimental and still in an early stage. A SHA-256 repository will in general not be able to share work with "regular" SHA-1
    /// repositories. It should be assumed that, e.g., Git internal file formats in relation to SHA-256 repositories may change in backwards-incompatible ways. Only use --object-format=sha256
    /// for testing purposes.
    #[arg(long, value_name="format", default_value="sha1")]
    pub object_format: HashAlgorithm,

    /// Use the specified name for the initial branch in the newly created repository. If not specified, fall back to the default name (currently master, but this is subject to change in the
    /// future; the name can be customized via the init.defaultBranch configuration variable).
    #[arg(short='b', long, value_name="branch-name", default_value="main")]
    pub initial_branch: String,

    /// Specify that the Git repository is to be shared amongst several users. This allows users belonging to the same group to push into that repository. When specified, the config variable
    /// "core.sharedRepository" is set so that files and directories under $GIT_DIR are created with the requested permissions. When not specified, Git will use permissions reported by umask(2).
    /// 
    /// The option can have the following values, defaulting to group if no value is given:
    ///     umask (or false)
    ///         Use permissions reported by umask(2). The default, when --shared is not specified.
    ///     group (or true)
    ///         Make the repository group-writable, (and g+sx, since the git group may be not the primary group of all users). This is used to loosen the permissions of an otherwise safe umask(2)
    ///         value. Note that the umask still applies to the other permission bits (e.g. if umask is 0022, using group will not remove read privileges from other (non-group) users). See 0xxx for
    ///         how to exactly specify the repository permissions.
    ///
    ///     all (or world or everybody)
    ///         Same as group, but make the repository readable by all users.
    /// 
    ///     <perm>
    ///         <perm> is a 3-digit octal number prefixed with ‘0` and each file will have mode <perm>.  <perm> will override users’ umask(2) value (and not only loosen permissions as group and all
    ///         does).  0640 will create a repository which is group-readable, but not group-writable or accessible to others.  0660 will create a repo that is readable and writable to the current
    ///         user and group, but inaccessible to others (directories and executable files get their x bit from the r bit for corresponding classes of users).
    #[arg(long, value_name="permissions", default_value="group", value_parser=parse_init_permission)]
    pub shared: InitPermissionFlag,

    /// If you provide a directory, the command is run inside it. If this directory does not exist, it will be created.
    pub directory: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_init_permission_should_parse_umask()
    {
        assert_eq!(parse_init_permission("umask"), Ok(InitPermissionFlag::Umask));
        assert_eq!(parse_init_permission("false"), Ok(InitPermissionFlag::Umask));
    }

    #[test]
    fn parse_init_permission_should_parse_group()
    {
        assert_eq!(parse_init_permission("group"), Ok(InitPermissionFlag::Group));
        assert_eq!(parse_init_permission("true"), Ok(InitPermissionFlag::Group));
    }

    #[test]
    fn parse_init_permission_should_parse_all()
    {
        assert_eq!(parse_init_permission("all"), Ok(InitPermissionFlag::All));
        assert_eq!(parse_init_permission("world"), Ok(InitPermissionFlag::All));
        assert_eq!(parse_init_permission("everybody"), Ok(InitPermissionFlag::All));
    }

    #[test]
    fn parse_init_permission_should_parse_octal()
    {
        assert_eq!(parse_init_permission("0022"), Ok(InitPermissionFlag::Perm(0o0022)));
    }

    #[test]
    fn parse_init_permission_should_error()
    {
        assert_eq!(parse_init_permission("notapermission"), Err(String::from("Invalid permission provided")));
        assert_eq!(parse_init_permission("0abc"), Err(String::from("Invalid permission provided")));
        assert_eq!(parse_init_permission("022"), Err(String::from("Invalid permission provided")));
        assert_eq!(parse_init_permission("1022"), Err(String::from("Invalid permission provided")));
        assert_eq!(parse_init_permission("02222"), Err(String::from("Invalid permission provided")));
    }

}
