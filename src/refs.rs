use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::error::RustGitError;

const REFS_FOLDER: &str = "refs";
const HEADS_FOLDER: &str = "heads";
const TAGS_FOLDER: &str = "tags";

pub(crate) struct GitRefs {
    refs_dir: PathBuf,
    heads_dir: PathBuf,
    tags_dir: PathBuf,
}

impl GitRefs {
    pub(crate) fn new(git_dir: &Path) -> Result<GitRefs, RustGitError> {
        let refs_dir = git_dir.join(REFS_FOLDER);
        let heads_dir = refs_dir.join(HEADS_FOLDER);
        let tags_dir = refs_dir.join(TAGS_FOLDER);

        fs::create_dir_all(&heads_dir)?;
        fs::create_dir_all(&tags_dir)?;

        Ok(GitRefs {
            refs_dir,
            heads_dir,
            tags_dir,
        })
    }

    pub(crate) fn try_read_ref(&self, ref_path: &Path) -> Result<Option<String>, RustGitError> {
        if fs::exists(&ref_path)? {
            return Ok(Some(fs::read_to_string(&ref_path)?));
        }
        Ok(None)
    }

    pub(crate) fn write_ref(&self, ref_path: &Path, new_value: &str) -> Result<(), RustGitError> {
        Ok(fs::write(&ref_path, new_value)?)
    }

    pub(crate) fn update_ref(
        &self,
        git_ref: &str,
        new_value: &str,
        old_value: Option<&str>,
    ) -> Result<(), RustGitError> {
        let ref_path = self.refs_dir.join(git_ref);
        match old_value {
            Some(old_value) => {
                let existing_value = self.try_read_ref(&ref_path)?;
                let existing_value = existing_value.unwrap_or(String::new());

                if old_value != existing_value {
                    return Err(RustGitError::new(format!("existing value '{existing_value}' for refs/{git_ref} doesn't match expected value '{old_value}'")));
                } else {
                    return Ok(self.write_ref(&ref_path, new_value)?);
                }
            }
            None => {
                return Ok(self.write_ref(&ref_path, new_value)?);
            }
        }
    }
}
