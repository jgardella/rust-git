
use std::{fs, path::{PathBuf, MAIN_SEPARATOR_STR}};

use crate::{command::GitCommand, index::GitIndexStageFlag, repo::{GitRepo, RepoState}, RustGitError};

use super::cli::MvArgs;

pub(crate) struct MvCommand {
    args: MvArgs,
}

impl MvCommand {
    pub fn new(args: MvArgs) -> MvCommand {
        MvCommand {
            args
        }
    }
}

fn get_src_and_dst(files: &Vec<String>) -> Result<(Vec<PathBuf>, PathBuf), RustGitError> {
    match files.as_slice() {
        [] => Err(RustGitError::new("mv expects at least 2 inputs, got 0")),
        [_] => Err(RustGitError::new("mv expects at least 2 inputs, got 1")),
        [source @ .., destination] => {
            Ok((source.iter().map(PathBuf::from).collect(), PathBuf::from(destination)))
        }
    }
}

struct MvAction {
    source: PathBuf,
    destination: PathBuf,
}

// There are a lot of checks done by C Git before any moves are done, but it's quite complex and using a lot
// of gotos: https://github.com/git/git/blob/master/builtin/mv.c#L252
//
// Here, I try to implement the checks that I can understand from that, but it's not one-to-one.
///
/// Checks that the provided source is valid for moving.
/// Returns the index of the entry in the index in case it is valid.
fn check_source(cmd: &MvCommand, repo: &GitRepo, source: &PathBuf, destination: &PathBuf) -> Result<Vec<MvAction>, String> {
    if cmd.args.dry_run {
        println!("Checking rename of '{source:?}' to '{destination:?}");
    }

    if let Ok(source_metadata) = fs::symlink_metadata(&source) {
        if let Ok(dest_metadata) = fs::symlink_metadata(&destination) {
            if source == destination && dest_metadata.is_dir() {
                return Err(String::from("can not move directory into itself"));
            } else if source_metadata.is_dir() {
                return Err(String::from("destination already exists"));
            } 
        }

        if source_metadata.is_dir() {
            let matching_entries = repo.index.entry_range_by_path(&source);

            if matching_entries.is_empty() {
                return Err(String::from("source directory is empty"));
            }

            return Ok(matching_entries.iter().map(|entry| {
                MvAction {
                    source: PathBuf::from(&entry.path_name),
                    destination: destination.join(&source),
                }
            }).collect());
        }

        if let Some(source_index_entry) = repo.index.entry_by_path(&source) {
            if source_index_entry.flags.stage != GitIndexStageFlag::RegularFileNoConflict {
                return Err(String::from("conflicted"));
            }

            if let Ok(_) = fs::symlink_metadata(&destination) {
                if cmd.args.force {
                    // only files can override each other:
                    // check both source and destination
                    if source.is_file() || source.is_symlink() {
                        if cmd.args.verbose {
                            println!("overwriting {destination:?}");
                        }
                    } else {
                        return Err(String::from("cannot overwrite"));
                    }
                } else {
                    return Err(String::from("cannot overwrite"));
                }
            } else {
                if destination.ends_with(MAIN_SEPARATOR_STR) {
                    return Err(String::from("destination directory does not exist"));
                }
            }
            return Ok(vec![MvAction {
                source: source.to_path_buf(),
                destination: destination.to_path_buf(),
            }]);
        }

        return Err(String::from("not under version control"));
    } else {
        if let Some(_) = repo.index.entry_by_path(source) {
            if let Some(_) = repo.index.entry_by_path(&destination) {
                if !cmd.args.force {
                    return Err(String::from("destination exists"));
                } 
            }
            return Ok(vec![MvAction {
                source: source.to_path_buf(),
                destination: destination.to_path_buf(),
            }]);
        } else {
            return Err(String::from("bad source"));
        }
    }
}

impl GitCommand for MvCommand {
    fn execute(&self, repo_state: RepoState) -> Result<(), RustGitError>
    {
        let mut repo = repo_state.try_get()?;

        let (sources, destination) = get_src_and_dst(&self.args.files)?;

        for source in sources {
            match check_source(&self, &repo, &source, &destination) {
                Ok(actions) => {
                    for action in actions {
                        if self.args.dry_run || self.args.verbose {
                            println!("Renaming {:?} to {:?}", action.source, action.destination);
                        }

                        if !self.args.dry_run {
                            match fs::rename(&action.source, &action.destination) {
                                Ok(_) => {
                                    repo.index.rename_entry_by_path(&action.source, &action.destination.to_str().unwrap());
                                },
                                Err(_) => {
                                    if !self.args.skip {
                                        return Err(RustGitError::new(format!("renaming {:?} failed", action.source)));
                                    }
                                },
                            }
                        }
                    }
                }
                Err(err) =>
                    if !self.args.skip {
                        return Err(RustGitError::new(String::from(format!("{err}, source={source:?}, destination={destination:?}"))));
                    }
            }
        }

        // TODO: cleanup empty source dirs:
        // https://github.com/git/git/blob/master/builtin/mv.c#L539

        repo.write_index()?;

        Ok(())
    }
}
