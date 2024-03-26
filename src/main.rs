mod repo;
mod command;
mod config;
mod error;
mod hash;
mod object;
mod options;
mod index;

mod init;
mod hash_object;
mod add;
mod cat_file;
mod update_index;

use std::{path::{Path, PathBuf}, process::ExitCode};

use add::cli::AddArgs;
use cat_file::cli::CatFileArgs;
use clap::{Parser, Subcommand};

use command::from_cli;
use error::RustGitError;
use hash_object::cli::HashObjectArgs;
use init::cli::InitArgs;
use repo::GitRepo;
use update_index::cli::UpdateIndexArgs;

fn parse_config_override(s: &str) -> Result<(String,String), String> {
    match s.find('=') {
        Some(eq_idx) => {
            Ok((String::from(&s[0..eq_idx]), String::from(&s[eq_idx+1..])))
        }
        None => {
            Ok((String::from(s), String::from("true")))
        }
    }
}

fn parse_config_env(s: &str) -> Result<(String,String), String> {
    match s.find('=') {
        Some(eq_idx) => {
            let env_var =  &s[eq_idx+1..];
            if env_var.is_empty() {
                Err(String::from("Expected env_var in config-env"))
            } else {
                Ok((String::from(&s[0..eq_idx]), String::from(env_var)))
            }
        }
        None => {
            Err(String::from("Expected '=' in config-env"))
        }
    }
}

#[derive(Debug, Parser)]
#[command(version)]
#[command(name = "rust-git")]
#[command(about = "An implementation of git in Rust", long_about = None)]
struct Cli {
    /// Run as if git was started in <path> instead of the current working
    /// directory. When multiple -C options are given, each subsequent
    /// non-absolute -C <path> is interpreted relative to the preceding -C
    /// <path>. If <path> is present but empty, e.g.  -C "", then the
    /// current working directory is left unchanged.
    ///
    /// This option affects options that expect path name like --git-dir
    /// and --work-tree in that their interpretations of the path names
    /// would be made relative to the working directory caused by the -C
    /// option. For example the following invocations are equivalent:
    ///
    ///    git --git-dir=a.git --work-tree=b -C c status
    ///    git --git-dir=c/a.git --work-tree=c/b status
    #[arg(short='C', value_name="path")]
    working_directory: Vec<PathBuf>,

    /// Pass a configuration parameter to the command. The value given will
    /// override values from configuration files. The <name> is expected in
    /// the same format as listed by git config (subkeys separated by
    /// dots).
    ///
    /// Note that omitting the = in git -c foo.bar ... is allowed and sets
    /// foo.bar to the boolean true value (just like [foo]bar would in a
    /// config file). Including the equals but with an empty value (like
    /// git -c foo.bar= ...) sets foo.bar to the empty string which git
    /// config --type=bool will convert to false.
    #[arg(short='c', value_parser=parse_config_override)]
    config_overrides: Vec<(String, String)>,

    /// Like -c <name>=<value>, give configuration variable <name> a value,
    /// where <envvar> is the name of an environment variable from which to
    /// retrieve the value. Unlike -c there is no shortcut for directly
    /// setting the value to an empty string, instead the environment
    /// variable itself must be set to the empty string. It is an error if
    /// the <envvar> does not exist in the environment.  <envvar> may not
    /// contain an equals sign to avoid ambiguity with <name> containing
    /// one.
    /// 
    /// This is useful for cases where you want to pass transitory
    /// configuration options to git, but are doing so on OS’s where other
    /// processes might be able to read your cmdline (e.g.
    /// /proc/self/cmdline), but not your environ (e.g.
    /// /proc/self/environ). That behavior is the default on Linux, but may
    /// not be on your system.
    /// 
    /// Note that this might add security for variables such as
    /// http.extraHeader where the sensitive information is part of the
    /// value, but not e.g.  url.<base>.insteadOf where the sensitive
    /// information can be part of the key.
    #[arg(long, value_parser=parse_config_env)]
    config_env: Vec<(String, String)>,

    /// Path to wherever your core Git programs are installed. This can
    /// also be controlled by setting the GIT_EXEC_PATH environment
    /// variable. If no path is given, git will print the current setting
    /// and then exit.
    #[arg(long, value_name="path", env="GIT_EXEC_PATH")]
    exec_path: Option<PathBuf>,

    /// Print the path, without trailing slash, where Git’s HTML
    /// documentation is installed and exit.
    #[arg(long)]
    html_path: bool,

    /// Print the manpath (see man(1)) for the man pages for this version
    /// of Git and exit.
    #[arg(long)]
    man_path: bool,

    /// Print the path where the Info files documenting this version of Git
    /// are installed and exit.
    #[arg(long)]
    info_path: bool,

    /// Pipe all output into less (or if set, $PAGER) if standard output is
    /// a terminal. This overrides the pager.<cmd> configuration options
    /// (see the "Configuration Mechanism" section below).
    #[arg(short, long)]
    paginate: bool,

    /// Do not pipe Git output into a pager.
    #[arg(short='P', long)]
    no_pager: bool,

    /// Set the path to the repository (".git" directory). This can also be
    /// controlled by setting the GIT_DIR environment variable. It can be
    /// an absolute path or relative path to current working directory.
    ///
    /// Specifying the location of the ".git" directory using this option
    /// (or GIT_DIR environment variable) turns off the repository
    /// discovery that tries to find a directory with ".git" subdirectory
    /// (which is how the repository and the top-level of the working tree
    /// are discovered), and tells Git that you are at the top level of the
    /// working tree. If you are not at the top-level directory of the
    /// working tree, you should tell Git where the top-level of the
    /// working tree is, with the --work-tree=<path> option (or
    /// GIT_WORK_TREE environment variable)
    /// 
    /// If you just want to run git as if it was started in <path> then use
    /// git -C <path>.
    #[arg(long, value_name="path", env="GIT_DIR")]
    git_dir: Option<PathBuf>,

    /// Set the path to the working tree. It can be an absolute path or a
    /// path relative to the current working directory. This can also be
    /// controlled by setting the GIT_WORK_TREE environment variable and
    /// the core.worktree configuration variable (see core.worktree in git-
    /// config(1) for a more detailed discussion).
    #[arg(long, value_name="path", env="GIT_WORK_TREE")]
    work_tree: Option<PathBuf>,

    /// Set the Git namespace. See gitnamespaces(7) for more details.
    /// Equivalent to setting the GIT_NAMESPACE environment variable.
    #[arg(long, value_name="path", env="GIT_NAMESPACE")]
    namespace: Option<PathBuf>,

    /// Currently for internal use only. Set a prefix which gives a path
    /// from above a repository down to its root. One use is to give
    /// submodules context about the superproject that invoked it.
    #[arg(long, value_name="path")]
    super_perfix: Option<PathBuf>,
    
    /// Treat the repository as a bare repository. If GIT_DIR environment
    /// is not set, it is set to the current working directory.
    #[arg(long)]
    bare: bool,

    /// Do not use replacement refs to replace Git objects. See git-
    /// replace(1) for more information.
    #[arg(long)]
    no_replace_objects: bool,

    /// Treat pathspecs literally (i.e. no globbing, no pathspec magic).
    /// This is equivalent to setting the GIT_LITERAL_PATHSPECS environment
    /// variable to 1.
    #[arg(long, env="GIT_LITERAL_PATHSPECS")]
    literal_pathspecs: bool,

    /// Add "glob" magic to all pathspec. This is equivalent to setting the
    /// GIT_GLOB_PATHSPECS environment variable to 1. Disabling globbing on
    /// individual pathspecs can be done using pathspec magic ":(literal)"
    #[arg(long, env="GIT_GLOB_PATHSPECS")]
    glob_pathspecs: bool,

    /// Add "literal" magic to all pathspec. This is equivalent to setting
    /// the GIT_NOGLOB_PATHSPECS environment variable to 1. Enabling
    /// globbing on individual pathspecs can be done using pathspec magic
    /// ":(glob)"
    #[arg(long, env="GIT_NOGLOB_PATHSPECS")]
    noglob_pathspecs: bool,

    /// Add "icase" magic to all pathspec. This is equivalent to setting
    /// the GIT_ICASE_PATHSPECS environment variable to 1.
    #[arg(long, env="GIT_ICASE_PATHSPECS")]
    icase_pathspecs: bool,

    /// Do not perform optional operations that require locks. This is
    /// equivalent to setting the GIT_OPTIONAL_LOCKS to 0.
    #[arg(long, env="GIT_OPTIONAL_LOCKS")]
    no_optional_locks: bool,

    /// List commands by group. This is an internal/experimental option and
    /// may change or be removed in the future. Supported groups are:
    /// builtins, parseopt (builtin commands that use parse-options), main
    /// (all commands in libexec directory), others (all other commands in
    /// $PATH that have git- prefix), list-<category> (see categories in
    /// command-list.txt), nohelpers (exclude helper commands), alias and
    /// config (retrieve command list from config variable
    /// completion.commands)
    #[arg(long, value_name="group")]
    list_cmds: Vec<String>,

    #[command(subcommand)]
    command: CliCommand,
}

#[derive(Debug, Subcommand)]
enum CliCommand {
    #[clap(alias="init-db")]
    Init(InitArgs),
    #[clap(alias="stage")]
    Add(AddArgs),
    HashObject(HashObjectArgs),
    CatFile(CatFileArgs),
    UpdateIndex(UpdateIndexArgs),
}

fn load_repo_and_execute(cli: Cli) -> Result<(), RustGitError> {
    // TODO: repo path should be determined based on args (git_dir, work_tree, etc)
    let repo_path = Path::new(".");
    let repo = GitRepo::new(repo_path)?;

    let command = from_cli(cli)?;
    command.execute(repo)
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    match load_repo_and_execute(cli) {
        Ok(_) => ExitCode::SUCCESS,
        Err(err) => {
            eprint!("{}", err.to_string());
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_config_override()
    {
        assert_eq!(parse_config_override("test=blah"), Ok((String::from("test"), String::from("blah"))));
        assert_eq!(parse_config_override("test="), Ok((String::from("test"), String::from(""))));
        assert_eq!(parse_config_override("test"), Ok((String::from("test"), String::from("true"))));
    }


    #[test]
    fn test_parse_config_env()
    {
        assert_eq!(parse_config_env("test=blah"), Ok((String::from("test"), String::from("blah"))));
        assert_eq!(parse_config_env("test="), Err(String::from("Expected env_var in config-env")));
        assert_eq!(parse_config_env("test"), Err(String::from("Expected '=' in config-env")));
    }
}
