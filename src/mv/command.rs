
use std::{fs, path::{Path, PathBuf, MAIN_SEPARATOR_STR}};

use crate::{command::GitCommand, index::GitIndexStageFlag, repo::{GitRepo, GitRepoPath, RepoState}, RustGitError};

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

fn get_src_and_dst(files: &Vec<String>) -> Result<(Vec<&str>, &str), RustGitError> {
    match files.as_slice() {
        [] => Err(RustGitError::new("mv expects at least 2 inputs, got 0")),
        [_] => Err(RustGitError::new("mv expects at least 2 inputs, got 1")),
        [source @ .., destination] => {
            Ok((source.iter().map(|s| s.as_str()).collect(), &destination))
        }
    }
}

struct MvAction {
    source: PathBuf,
    destination: PathBuf,
    update_index: bool,
    update_working_dir: bool,
}

// There are a lot of checks done by C Git before any moves are done, but it's quite complex and using a lot
// of gotos: https://github.com/git/git/blob/master/builtin/mv.c#L252
//
// Here, I try to implement the checks that I can understand from that, but it's not one-to-one.
///
/// Checks that the provided source is valid for moving.
/// Returns the index of the entry in the index in case it is valid.
fn check_source(cmd: &MvCommand, repo: &GitRepo, source: &str, destination: &str, source_repo_path: &GitRepoPath, destination_repo_path: &GitRepoPath) -> Result<Vec<MvAction>, String> {
    if cmd.args.dry_run {
        println!("Checking rename of {source:?} to {destination:?}");
    }

    if let Ok(source_metadata) = fs::symlink_metadata(&source) {
        let existing_dir = 
            if let Ok(dest_metadata) = fs::symlink_metadata(&destination) {
                if source == destination && dest_metadata.is_dir() {
                    return Err(String::from("can not move directory into itself"));
                } 
                if source_metadata.is_dir() && dest_metadata.is_file() {
                    return Err(String::from("destination already exists"));
                } 
                dest_metadata.is_dir()
            } else {
                false
            };

        let destination_prefix =
            if existing_dir {
                let source_path_buf = PathBuf::from(&source);
                let src_dir_name = source_path_buf.file_name().unwrap();
                PathBuf::from(&destination).join(&src_dir_name)
            } else {
                PathBuf::from(&destination)
            };

        if source_metadata.is_dir() {
            let matching_entries = repo.index.entry_range_by_path(&source_repo_path);

            if matching_entries.is_empty() {
                return Err(String::from("source directory is empty"));
            }
            
            let mut actions = Vec::new();

            actions.push(MvAction {
                source: PathBuf::from(&source),
                destination: destination_prefix.to_path_buf(),
                update_index: false,
                update_working_dir: true,
            });

            for entry in matching_entries.iter() {
                let index_path_name = entry.path_name.as_path_buf();
                let index_file_name = &entry.path_name.as_string()[source.len()+1..];
                let new_destination = destination_prefix.join(&index_file_name);

                actions.push(MvAction {
                    source: index_path_name,
                    destination: new_destination,
                    update_index: true,
                    update_working_dir: false, // Actual files will already be moved by moving the parent directory.
                });
            }

            return Ok(actions);
        }

        if let Some((_, source_index_entry)) = repo.index.entry_by_path(&source_repo_path) {
            if source_index_entry.flags.stage != GitIndexStageFlag::RegularFileNoConflict {
                return Err(String::from("conflicted"));
            }

            if let Ok(_) = fs::symlink_metadata(&destination) {
                if existing_dir {
                    return Ok(vec![MvAction {
                        source: PathBuf::from(source),
                        destination: destination_prefix,
                        update_index: true,
                        update_working_dir: true,
                    }]);
                }

                if cmd.args.force {
                    // only files can override each other:
                    // check both source and destination
                    if source_metadata.is_file() || source_metadata.is_symlink() {
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
                source: PathBuf::from(source),
                destination: PathBuf::from(destination),
                update_index: true,
                update_working_dir: true,
            }]);
        }

        return Err(String::from("not under version control"));
    } else {
        if let Some(_) = repo.index.entry_by_path(&source_repo_path) {
            if let Some(_) = repo.index.entry_by_path(&destination_repo_path) {
                if !cmd.args.force {
                    return Err(String::from("destination exists"));
                } 
            }
            return Ok(vec![MvAction {
                source: PathBuf::from(source),
                destination: PathBuf::from(destination),
                update_index: true,
                update_working_dir: false,
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

        let destination_repo_path = repo.path_to_git_repo_path(Path::new(destination))?;

        for source in sources {
            let source_repo_path = repo.path_to_git_repo_path(Path::new(source))?;
            match check_source(&self, &repo, &source, &destination, &source_repo_path, &destination_repo_path) {
                Ok(actions) => {
                    for action in actions {
                        if self.args.dry_run || self.args.verbose {
                            println!("Renaming {:?} to {:?}", action.source, action.destination);
                        }

                        if !self.args.dry_run {
                            if action.update_working_dir {
                                match fs::rename(&action.source, &action.destination) {
                                    Ok(_) => (),
                                    Err(err) => {
                                        if !self.args.skip {
                                            return Err(RustGitError::new(format!("renaming {:?} failed: {:?}", action.source, err)));
                                        }
                                    },
                                }
                            }

                            if action.update_index {
                                let source_repo_path = repo.path_to_git_repo_path(&action.source)?;
                                let destination_repo_path = repo.path_to_git_repo_path(&action.destination)?;
                                repo.index.rename_entry_by_path(&source_repo_path, &destination_repo_path);
                            }
                        }
                    }
                }
                Err(err) => {
                    if !self.args.skip {
                        return Err(RustGitError::new(String::from(format!("{err}, source={source:?}, destination={destination:?}"))));
                    }
                }
            }
        }

        // TODO: cleanup empty source dirs:
        // https://github.com/git/git/blob/master/builtin/mv.c#L539

        repo.write_index()?;

        Ok(())
    }
}
