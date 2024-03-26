use std::{fs::File, io::{self, BufRead, BufReader}};
use crate::{command::GitCommand, repo::RepoState, RustGitError};

use super::cli::HashObjectArgs;

pub(crate) struct HashObjectCommand {
    args: HashObjectArgs,

    // TODO: add base args
}

impl HashObjectCommand {
    pub fn new(args: HashObjectArgs) -> HashObjectCommand {
        HashObjectCommand {
            args
        }
    }
}

fn collect_items_to_hash(cmd: &HashObjectCommand) -> Result<Vec<Box<dyn BufRead>>, RustGitError> {
    let mut to_hash: Vec<Box<dyn BufRead>> = Vec::new();
    
    // Add input from stdin.
    if cmd.args.stdin_paths {
        let stdin_files: Vec<File> =
            io::stdin().lock().lines()
            .map(|filename| filename.and_then(File::open))
            .collect::<std::io::Result<_>>()?;

        stdin_files.into_iter().for_each(|file| to_hash.push(Box::new(BufReader::new(file))));
    }
    else if cmd.args.stdin {
        let stdin_reader = Box::new(BufReader::new(io::stdin()));
        to_hash.push(stdin_reader);
    }

    // Even if --stdin is specified, C Git still hashes the provided filenames:
    // https://github.com/git/git/blob/master/builtin/hash-object.c#L153-L165
    let files: Vec<File> = 
        cmd.args.files
        .iter()
        .map(File::open)
        .collect::<io::Result<_>>()?;

    files.into_iter().for_each(|file| to_hash.push(Box::new(BufReader::new(file))));

    return Ok(to_hash);
}

impl GitCommand for HashObjectCommand {
    fn execute(&self, repo_state: RepoState) -> Result<(), RustGitError>
    {
        let mut repo = repo_state.try_get()?;

        // Omitting implementaion of --no-filters, --path, and --literally for now, for simplicity
        if self.args.no_filters {
            return Err(RustGitError::new(String::from("--no-filters not supported")));
        }

        if self.args.path.is_some() {
            return Err(RustGitError::new(String::from("--path not supported")));
        }

        if self.args.literally {
            return Err(RustGitError::new(String::from("--literally not supported")));
        }

        let mut to_hash = collect_items_to_hash(&self)?;

        to_hash.iter_mut().map(|br| {
            let mut s = String::new();
            br.read_to_string(&mut s)?;
            repo.index(self.args.object_type, s, self.args.write)
            .map(|object_id| println!("{}", object_id))
        }).collect::<Result<(), RustGitError>>()
    }
}
