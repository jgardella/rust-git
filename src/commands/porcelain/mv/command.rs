use std::path::{Path, MAIN_SEPARATOR_STR};

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
    source: GitRepoPath,
    destination: GitRepoPath,
    update_index: bool,
    update_working_dir: bool,
}

#[derive(PartialEq)]
enum DestinationType {
    ExistingDirectory,
    ExistingFile,
    MissingFile,
    MissingDirectory,
}

// There are a lot of checks done by C Git before any moves are done, but it's quite complex and using a lot
// of gotos: https://github.com/git/git/blob/master/builtin/mv.c#L252
//
// Here, I try to implement the checks that I can understand from that, but it's not one-to-one.
///
/// Checks that the provided source is valid for moving.
/// Returns the index of the entry in the index in case it is valid.
fn check_source(cmd: &MvCommand, repo: &GitRepo, source: GitRepoPath, destination: GitRepoPath, destination_str: &str) -> Result<Vec<MvAction>, String> {
    if cmd.args.dry_run {
        println!("Checking rename of {source} to {destination}");
    }

    if let Ok(source_metadata) = repo.symlink_metadata(&source) {
        let destination_type = 
            if let Ok(dest_metadata) = &repo.symlink_metadata(&destination) {
                if source == destination && dest_metadata.is_dir() {
                    return Err(String::from("can not move directory into itself"));
                } 
                if source_metadata.is_dir() && dest_metadata.is_file() {
                    return Err(String::from("destination already exists"));
                } 
                if dest_metadata.is_dir() {
                    DestinationType::ExistingDirectory
                } else {
                    DestinationType::ExistingFile
                }
            } else {
                if destination_str.ends_with(MAIN_SEPARATOR_STR) {
                    DestinationType::MissingDirectory

                } else {
                    DestinationType::MissingFile
                }
            };

        let full_destination =
            if destination_type == DestinationType::ExistingDirectory {
                source.as_moved_file(&destination)
            } else {
                destination
            };

        if source_metadata.is_dir() {
            let matching_entries = repo.index.entry_range_by_path(&source);

            if matching_entries.is_empty() {
                return Err(String::from("source directory is empty"));
            }
            
            let mut actions = Vec::new();

            actions.push(MvAction {
                source: source,
                destination: full_destination.clone(),
                update_index: false,
                update_working_dir: true,
            });

            for entry in matching_entries.iter() {
                let new_destination = entry.path_name.as_moved_file(&full_destination);

                actions.push(MvAction {
                    source: entry.path_name.clone(),
                    destination: new_destination,
                    update_index: true,
                    update_working_dir: false, // Actual files will already be moved by moving the parent directory.
                });
            }

            return Ok(actions);
        }

        if let Some((_, source_index_entry)) = repo.index.entry_by_path(&source) {
            if source_index_entry.flags.stage != GitIndexStageFlag::RegularFileNoConflict {
                return Err(String::from("conflicted"));
            }

            match destination_type {
                DestinationType::ExistingDirectory =>
                    return Ok(vec![MvAction {
                        source: source,
                        destination: full_destination,
                        update_index: true,
                        update_working_dir: true,
                    }]),
                DestinationType::ExistingFile => {
                    if cmd.args.force {
                        // only files can override each other:
                        // check both source and destination
                        if source_metadata.is_file() || source_metadata.is_symlink() {
                            if cmd.args.verbose {
                                println!("overwriting {full_destination:?}");
                            }

                            return Ok(vec![MvAction {
                                source: source,
                                destination: full_destination,
                                update_index: true,
                                update_working_dir: true,
                            }]);
                        } else {
                            return Err(String::from("cannot overwrite"));
                        }
                    } else {
                        return Err(String::from("cannot overwrite"));
                    }
                },
                DestinationType::MissingDirectory => {
                    return Err(String::from("destination directory does not exist"));
                },
                DestinationType::MissingFile =>
                    return Ok(vec![MvAction {
                        source: source,
                        destination: full_destination,
                        update_index: true,
                        update_working_dir: true,
                    }])
            };
        }

        return Err(String::from("not under version control"));
    } else {
        if let Some(_) = repo.index.entry_by_path(&source) {
            if let Some(_) = repo.index.entry_by_path(&destination) {
                if !cmd.args.force {
                    return Err(String::from("destination exists"));
                } 
            }
            return Ok(vec![MvAction {
                source: source,
                destination: destination,
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
            match check_source(&self, &repo, source_repo_path, destination_repo_path.clone(), &destination) {
                Ok(actions) => {
                    for action in actions {
                        if self.args.dry_run || self.args.verbose {
                            println!("Renaming {} to {}", action.source, action.destination);
                        }

                        if !self.args.dry_run {
                            if action.update_working_dir {
                                match repo.rename_file(&action.source, &action.destination) {
                                    Ok(_) => (),
                                    Err(err) => {
                                        if !self.args.skip {
                                            return Err(RustGitError::new(format!("renaming {} failed: {:?}", action.source, err)));
                                        }
                                    },
                                }
                            }

                            if action.update_index {
                                repo.index.rename_entry_by_path(&action.source, &action.destination);
                            }
                        }
                    }
                }
                Err(err) => {
                    if !self.args.skip {
                        return Err(RustGitError::new(String::from(format!("{err}, source=\"{source}\", destination=\"{destination}\""))));
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
