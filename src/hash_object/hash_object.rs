use std::{fs::File, io::{self, BufRead, BufReader}, os::fd::{AsFd, BorrowedFd}};
use crate::RustGitError;

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

fn collect_items_to_hash(cmd: &HashObjectCommand) -> Vec<Box<dyn BufRead>> {
    let mut to_hash: Vec<Box<dyn BufRead>> = Vec::new();
    
    // Add input from stdin.
    if cmd.args.stdin_paths {
        io::stdin().lock().lines()
        .for_each(|filename| {
            match filename.and_then(File::open).map(BufReader::new) {
                Ok(buf_reader) => to_hash.push(Box::new(buf_reader)),
                Err(err) => () // TODO: currently skipping files that couldn't be opened. What does C Git do?
            }
        })
    }
    else if cmd.args.stdin {
        to_hash.push(Box::new(BufReader::new(io::stdin())))
    }

    // Even if --stdin is specified, C Git still hashes the provided filenames:
    // https://github.com/git/git/blob/master/builtin/hash-object.c#L153-L165
    cmd.args.files
    .iter()
    .for_each(|file| {
        match File::open(file).map(BufReader::new) {
            Ok(buf_reader) => to_hash.push(Box::new(buf_reader)),
            Err(err) => () // TODO: currently skipping files that couldn't be opened. What does C Git do?
        }
    });

    return to_hash;
}

pub(crate) fn hash_object(cmd: &HashObjectCommand) -> Result<(), RustGitError> // TODO: figure out return type
{
    let mut to_hash = collect_items_to_hash(cmd);

    to_hash.iter_mut().for_each(|br| {
        let mut s = String::new();
        br.read_to_string(&mut s);
        println!("{}", s)
    });
    
    Ok(())
}
