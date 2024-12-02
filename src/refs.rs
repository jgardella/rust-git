use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::{error::RustGitError, object::GitObjectId};

const REFS_FOLDER: &str = "refs";
const HEADS_FOLDER: &str = "heads";
const TAGS_FOLDER: &str = "tags";

pub(crate) struct GitRefs {
    git_dir: PathBuf,
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
            git_dir: git_dir.to_path_buf(),
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

    pub(crate) fn get_symbolic_ref(&self, ref_name: &str) -> Result<Option<String>, RustGitError> {
        let path = self.git_dir.join(ref_name);
        if !fs::exists(&path)? {
            return Ok(None);
        }

        let value = fs::read_to_string(&path)?;

        return Ok(Some(value));
    }

    pub(crate) fn update_symbolic_ref(
        &self,
        ref_name: &str,
        new_value: &str,
    ) -> Result<(), RustGitError> {
        let path = self.git_dir.join(ref_name);
        fs::write(path, format!("ref: {new_value}"))?;
        Ok(())
    }

    pub(crate) fn delete_symbolic_ref(&self, ref_name: &str) -> Result<(), RustGitError> {
        let path = self.git_dir.join(ref_name);
        if !fs::exists(&path)? {
            return Ok(());
        }

        let value = fs::read_to_string(&path)?;

        if value.starts_with("ref: ") {
            fs::remove_file(&path)?;
            Ok(())
        } else {
            Err(RustGitError::new(format!(
                "cannot delete detatched symbolic-ref '{ref_name}'"
            )))
        }
    }

    pub(crate) fn tag_path(&self, tag_name: &str) -> PathBuf {
        self.tags_dir.join(tag_name)
    }

    pub(crate) fn try_read_tag(&self, tag_name: &str) -> Result<Option<String>, RustGitError> {
        let tag_path = self.tag_path(tag_name);
        self.try_read_ref(&tag_path)
    }

    pub(crate) fn get_head_ref(
        &self,
    ) -> Result<(Option<String>, Option<GitObjectId>), RustGitError> {
        if let Some(head_ref_value) = self.get_symbolic_ref("HEAD")? {
            if !head_ref_value.starts_with("ref: ") {
                return Err(RustGitError::new("HEAD is in detatched state"));
            }

            let head_ref_branch = head_ref_value.trim_start_matches("ref: ");

            let ref_path = self.git_dir.join(head_ref_branch);

            let ref_id = self.try_read_ref(&ref_path)?.map(GitObjectId::new);
            Ok((Some(head_ref_branch.to_string()), ref_id))
        } else {
            Ok((None, None))
        }
    }

    pub(crate) fn create_tag(
        &self,
        tag_name: &str,
        object_id: &GitObjectId,
    ) -> Result<(), RustGitError> {
        let tag_path = self.tag_path(tag_name);
        self.write_ref(&tag_path, &object_id.to_string())
    }

    pub(crate) fn delete_tag(&self, tag_name: &str) -> Result<(), RustGitError> {
        let tag_path = self.tag_path(tag_name);
        if !fs::exists(&tag_path)? {
            return Err(RustGitError::new(format!("no tag {tag_name}")));
        }

        return Ok(fs::remove_file(&tag_path)?);
    }

    pub(crate) fn list_tags(&self) -> Result<Vec<String>, RustGitError> {
        let tag_files = fs::read_dir(&self.tags_dir)?;

        let mut tags = Vec::new();
        for tag_file in tag_files {
            if let Some(tag_name) = tag_file?.file_name().to_str() {
                tags.push(tag_name.to_string())
            } else {
                return Err(RustGitError::new("invalid tag name"));
            }
        }

        Ok(tags)
    }
}
