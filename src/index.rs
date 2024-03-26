/// All binary numbers are in network byte order.

use std::{fs::{File, Metadata}, io::Write, os::unix::fs::{MetadataExt, PermissionsExt}, path::{Path, PathBuf}};

use serde::{Deserialize, Serialize};

use crate::{error::RustGitError, object::GitObjectId};

const DEFAULT_INDEX_NAME: &str = "index";

#[derive(Serialize, Deserialize)]
pub(crate) enum GitIndexVersion {
    V2,
    V3,
    V4,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct GitIndexHeader {
    version: GitIndexVersion,
    num_entries: u32,
}

#[derive(Serialize, Deserialize)]
pub(crate) enum GitIndexObjectType {
    RegularFile,
    SymbolicLink,
    GitLink,
}

#[derive(Serialize, Deserialize)]
pub (crate) struct GitIndexTimestamp {
    seconds: u32,
    nanoseconds: u32,
}

#[derive(Serialize, Deserialize)]
pub (crate) enum GitIndexUnixPermission {
    Permission0755,
    Permission0644,
    None,
}

/// Index entries are sorted in ascending order on the name field,
/// interpreted as a string of unsigned bytes (i.e. memcmp() order, no
/// localization, no special casing of directory separator '/'). Entries
/// with the same name are sorted by their stage field.
#[derive(Serialize, Deserialize)]
pub(crate) struct GitIndexEntry {
    last_metadata_update: GitIndexTimestamp,
    last_data_update: GitIndexTimestamp,
    dev: u32,
    ino: u32,
    uid: u32,
    gid: u32,
    file_size: u32,
    name: GitObjectId,

    // mode
    obj_type: GitIndexObjectType,
    unix_permission: GitIndexUnixPermission,

    // flags
    assume_valid: bool,
    extended: bool,
    stage: u32,

    /// Entry path name (variable length) relative to top level directory
    /// (without leading slash). '/' is used as path separator. The special
    /// path components ".", ".." and ".git" (without quotes) are disallowed.
    /// Trailing slash is also disallowed.
    path_name: String,
}

impl GitIndexEntry {
    pub(crate) fn new(path: &str, metadata: &Metadata, obj_id: GitObjectId) -> GitIndexEntry {
        let (obj_type, unix_permission) =
            if metadata.is_symlink() {
                (GitIndexObjectType::SymbolicLink, GitIndexUnixPermission::None)
            } else if metadata.is_dir() {
                (GitIndexObjectType::GitLink, GitIndexUnixPermission::None)
            } else {
                // If file is executable by owner, we set 755, otherwise 644.
                let unix_permission = 
                    if metadata.permissions().mode() | 0b0100 != 0 {
                        GitIndexUnixPermission::Permission0755
                    } else {
                        GitIndexUnixPermission::Permission0644
                    };
                (GitIndexObjectType::RegularFile, unix_permission)
            };


        GitIndexEntry {
            last_metadata_update:  GitIndexTimestamp {
                seconds: metadata.ctime() as u32,
                nanoseconds: metadata.ctime_nsec() as u32,
            },
            last_data_update:  GitIndexTimestamp {
                seconds: metadata.mtime() as u32,
                nanoseconds: metadata.mtime_nsec() as u32,
            },
            dev: metadata.dev() as u32,
            ino: metadata.ino() as u32,
            uid: metadata.uid() as u32,
            gid: metadata.gid() as u32,
            file_size: metadata.size() as u32,
            name: obj_id,
            obj_type,
            unix_permission,
            assume_valid: false,
            extended: false,
            stage: 0,
            path_name: String::from(path),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub(crate) struct GitIndex {
    header: GitIndexHeader,
    entries: Vec<GitIndexEntry>,
}

impl Default for GitIndex {
    fn default() -> Self {
        Self { 
            header: GitIndexHeader {
                version: GitIndexVersion::V2,
                num_entries: 0,
            },
            entries: vec![],
        }
    }
}

impl GitIndex {
    /// Loads the index from the provided git directory.
    pub(crate) fn open(git_dir: &Path) -> Result<GitIndex, RustGitError> {
        let index_file_path: PathBuf = git_dir.join(DEFAULT_INDEX_NAME);

        if !index_file_path.exists() {
            return Ok(GitIndex::default());
        }

        // Reference for index file format: https://git-scm.com/docs/index-format
        let index_file = File::open(index_file_path)?;

        // TODO: implement custom serde for Git index file binary format
        let git_index: GitIndex = bincode::deserialize_from(&index_file)?;

        Ok(git_index)
    }

    pub(crate) fn write(&mut self, git_dir: &Path) -> Result<(), RustGitError> {
        let mut index_file = File::create(&git_dir.join(DEFAULT_INDEX_NAME))?;

        let serialized = bincode::serialize(&self)?;

        index_file.write(&serialized)?;

        Ok(())
    }

    pub(crate) fn add(&mut self, index_entry: GitIndexEntry) {
        // Replace existing index entry with same path & stage.
        let existing_entry = self.entries.iter_mut().find(|item| item.path_name == index_entry.path_name && item.stage == index_entry.stage);

        match existing_entry {
            Some(existing) => *existing = index_entry,
            None => self.entries.push(index_entry)
        }
    }

    pub(crate) fn try_find_by_path(&self, path: &str) -> Option<&GitIndexEntry> {
        // TODO: make this more efficient
        self.entries.iter().find(|item| item.path_name == path)
    }
}
