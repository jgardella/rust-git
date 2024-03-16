mod init;

use std::path::PathBuf;

use clap::{Parser, Subcommand};

use init::cli::InitArgs;

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

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Init(InitArgs)
}


fn main() {
    let cli = Cli::parse();
    println!("{:?}", cli);

    // You can check for the existence of subcommands, and if found use their
    // matches just as you would the top level cmd
    match &cli.command {
        Some(Commands::Init(_)) => {}
        None => {}
    }

    // Continued program logic goes here...
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
}
