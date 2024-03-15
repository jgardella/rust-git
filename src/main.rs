mod init;
use clap::{Parser, Subcommand};

use crate::init::cli::InitArgs;

#[derive(Parser)]
#[command(version)]
#[command(name = "rust-git")]
#[command(about = "An implementation of git in Rust", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Init(InitArgs)
}


fn main() {
    let cli = Cli::parse();

    // You can check for the existence of subcommands, and if found use their
    // matches just as you would the top level cmd
    match &cli.command {
        Some(Commands::Init(init_args)) => {
            println!("{:?}", init_args);
        }
        None => {}
    }

    // Continued program logic goes here...
}
